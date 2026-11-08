import type { UiActions } from '../../app/UiActions'
import { ANALYSIS_ADVICE, STARTUP_ADVICE, adviceIn, analysisPrompt, fixedAdvice, startupPrompt } from '../../core/assistant/advicePrompts'
import { Conversation } from '../../core/assistant/Conversation'
import { chatPrompt, leftoversPrompt } from '../../core/assistant/prompts'
import { chatReport } from '../../core/assistant/report'
import { adviceSchema, leftoverSchema, parseItems, withAdvice } from '../../core/assistant/schemas'
import type { Analysis, CommandError, StartupEntry } from '../../core/model/types'
import { tx } from '../../core/text/format'

/**
 * The local model, loaded only on demand. It describes items and answers questions; it never
 * decides what gets deleted and never runs anything.
 */
export class AssistantService {
  private ready = false
  private conversation: Conversation | null = null

  constructor(private readonly ui: UiActions) {}

  /** New scan data: the next question starts a new conversation with a fresh report. */
  reset(): void {
    this.conversation = null
  }

  private get model(): string {
    return this.ui.store.get().info.model
  }

  /** Starts the private server and checks the model; false (with a log line) when unavailable. */
  private async ensure(): Promise<boolean> {
    const s = this.ui.store.get()
    if (s.info.noAi) {
      this.ui.line(tx('The AI model is disabled (started with -BezLLM).'), 'yellow')
      return false
    }
    if (this.ready) return true
    this.ui.line(tx('Starting model {0} (the first use takes about 10-20 s)...', this.model), 'darkGray')
    try {
      await this.ui.api.aiPrepare(this.model)
      this.ready = true
    } catch (e) {
      this.ui.line((e as CommandError).message, 'yellow')
    }
    return this.ready
  }

  private async describe(prompt: string, list: object[], schema: object, field: 'explanations' | 'items') {
    const answer = await this.ui.api.aiChat(this.model, [{ role: 'system', content: prompt }, { role: 'user', content: JSON.stringify(list) }], schema)
    return parseItems(answer, field)
  }

  /** Runs a description job; returns the count or -1 when the assistant is unavailable or failed. */
  private async job(count: number, work: () => Promise<number>): Promise<number> {
    if (!(await this.ensure())) return -1
    this.ui.line(tx('The model is describing {0} items (about 20-40 s)...', String(count)), 'darkGray')
    try {
      const n = await work()
      this.ui.line(tx('Done - {0} items described.', String(n)), 'green')
      this.reset()
      return n
    } catch (e) {
      this.ui.line(tx('Model error: {0}', (e as CommandError).detail ?? String(e)), 'yellow')
      return -1
    }
  }

  /** What each leftover was, 30 at a time (a whole big list does not fit in the model's context). */
  async describeLeftovers(): Promise<number> {
    const left = this.ui.store.get().report?.leftovers ?? []
    if (!left.length) {
      this.ui.line(tx('No leftovers to explain.'), 'darkGray')
      return 0
    }
    const list = left.map((l) => ({ id: l.id, type: l.kind, name: l.name, path: l.path, contains: l.subkeys }))
    return this.job(list.length, async () => {
      let n = 0
      for (let i = 0; i < list.length; i += 30) {
        const items = await this.describe(leftoversPrompt(this.ui.tr), list.slice(i, i + 30), leftoverSchema(), 'explanations')
        const ai = { ...this.ui.store.get().ai }
        for (const it of items.filter((x) => left.some((l) => l.id === x.id))) {
          ai[it.id] = it.description
          n++
        }
        this.ui.store.set({ ai })
      }
      return n
    })
  }

  async describeAnalysis(a: Analysis): Promise<number> {
    const tr = this.ui.tr
    const all = [...a.folders, ...a.files]
    const list = all.map((i) => ({ id: i.id, type: i.kind, path: i.path, size: tr.size(i.bytes), hint: i.hint ? tr.text(i.hint) : null, last_modified: i.lastWrite }))
    const advice = adviceIn(ANALYSIS_ADVICE, tr)
    return this.job(list.length, async () => {
      const items = await this.describe(analysisPrompt(tr), list, adviceSchema(advice), 'items')
      return this.apply(items, all.map((i) => i.id), (id, given) => {
        const fixed = fixedAdvice(all.find((i) => i.id === id)?.path ?? '')
        return fixed !== null ? advice[fixed] : given && advice.includes(given) ? given : advice[5]
      })
    })
  }

  async describeStartup(entries: StartupEntry[]): Promise<number> {
    const tr = this.ui.tr
    if (!entries.length) return 0
    const list = entries.map((e) => ({ id: e.id, name: e.name, command: e.command, publisher: e.publisher, source: tr.text(e.source) }))
    const advice = adviceIn(STARTUP_ADVICE, tr)
    return this.job(list.length, async () => {
      const items = await this.describe(startupPrompt(tr), list, adviceSchema(advice), 'items')
      // Protected entries always get "keep it on", whatever the model said
      return this.apply(items, entries.map((e) => e.id), (id, given) =>
        entries.find((e) => e.id === id)?.protected ? advice[0] : given && advice.includes(given) ? given : advice[2],
      )
    })
  }

  private apply(items: Array<{ id: string; description: string; advice?: string }>, ids: string[], pick: (id: string, given?: string) => string): number {
    const ai = { ...this.ui.store.get().ai }
    let n = 0
    for (const it of items.filter((x) => ids.includes(x.id))) {
      ai[it.id] = withAdvice(it.description, this.ui.tr.t('Advice'), pick(it.id, it.advice))
      n++
    }
    this.ui.store.set({ ai })
    return n
  }

  /** A chat question; null when there is no answer (the reason is in the Log). */
  async ask(question: string): Promise<string | null> {
    if (!(await this.ensure())) return null
    const s = this.ui.store.get()
    if (!this.conversation) {
      // Chat works before a scan too - the model then knows it does not know this computer's files
      const report = s.report
        ? chatReport({ categories: s.report.categories, leftovers: s.report.leftovers, selected: s.selected, profile: s.report.profile, analyses: Object.values(s.analyses), startup: s.startup ?? [], ai: s.ai }, s.tr)
        : s.tr.t('No scan has been run yet - you do not know the files on this computer. Answer in general terms; for questions about specific files, folders or the plan, ask the user to click "Scan" first.')
      this.conversation = new Conversation(`${chatPrompt(s.tr)}\n\n${s.tr.t('REPORT AND PLAN')}:\n${report}`)
    }
    try {
      const answer = await this.ui.api.aiChat(this.model, this.conversation.request(question))
      if (!answer.trim()) throw { message: tx('the model returned an empty answer'), detail: 'empty' }
      this.conversation.record(question, answer)
      return answer
    } catch (e) {
      const err = e as CommandError
      this.ui.line(tx('Model error: {0}', err.detail ?? String(e)), 'yellow')
      return null
    }
  }

  /** Frees VRAM and RAM (before cleaning, after installing a model). */
  async stop(): Promise<void> {
    this.ready = false
    await this.ui.api.aiStop().catch(() => undefined)
  }
}
