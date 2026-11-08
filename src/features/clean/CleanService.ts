import type { UiActions } from '../../app/UiActions'
import { applyStep, doneCount, initialSteps } from '../../core/plan/steps'
import { summary } from '../../core/plan/summary'
import type { StepState, Text } from '../../core/model/types'
import { tx } from '../../core/text/format'

/** The cleanup: summary and confirmation (default answer: no), then the progress screen. */
export class CleanService {
  constructor(
    private readonly ui: UiActions,
    private readonly afterClean: () => Promise<void>,
  ) {}

  async loadHistory(): Promise<void> {
    const history = await this.ui.api.history().catch(() => [])
    this.ui.store.set({ history })
  }

  /** Summary in a dialog; cleaning starts only after "Yes". */
  async confirmAndClean(): Promise<void> {
    const { store, tr } = this.ui
    const s = store.get()
    if (!s.report || s.info.preview || s.cleaned) return
    const lines = summary(s.report.categories, s.report.leftovers, s.selected, s.report.freeBefore, s.info.systemDrive)
    if (!lines) {
      await this.ui.message(tr.t('Nothing is selected - there is nothing to clean.'))
      return
    }
    const yes = await this.ui.confirm({
      title: tr.t('PC Cleaner - summary before cleaning'),
      lines: lines.map((l) => ({ text: tr.text(l.text), tone: l.tone, value: l.value ? tr.text(l.value) : undefined })),
      question: tr.t('Start cleaning?'),
      tone: 'warning',
    })
    if (yes) await this.run()
  }

  /** Called from the step events while the cleaner works. */
  onStep(id: string, state: StepState, bytes: number, note: Text | null): void {
    const steps = applyStep(this.ui.store.get().steps, id, state, bytes, note)
    this.ui.store.set({ steps })
    if (this.ui.store.get().cleaning && steps.length) void this.ui.api.setProgress('normal', (100 * doneCount(steps)) / steps.length)
  }

  /** Cleans the current selection (the automatic cleaning calls this without a dialog). */
  async run(): Promise<boolean> {
    const { store, api } = this.ui
    const r = store.get().report
    if (!r) return false
    const sel = store.get().selected
    store.set({ steps: initialSteps(r.categories, r.leftovers, sel), cleanResult: null, cleaning: true })
    this.ui.go('cleanup')
    this.ui.line(tx('2/2  Cleaning'), 'cyan')
    const cats = r.categories.filter((c) => sel.has(c.id)).map((c) => c.id)
    const lefts = r.leftovers.filter((l) => sel.has(l.id)).map((l) => l.id)
    const result = await this.ui.busy(() => api.clean(cats, lefts))
    store.set({ cleaning: false })
    if (!result) {
      store.set((st) => ({ steps: st.steps.map((x) => (x.state === 'running' ? { ...x, state: 'fail', note: tx('interrupted - see the Log') } : x)) }))
      void api.setProgress('error', 100)
      return false
    }
    void api.setProgress('none')
    store.set({ cleaned: true, cleanResult: result })
    const ms = result.durationMs
    this.ui.line(tx('Data removed: {0}', { size: result.total }), 'white')
    this.ui.line(tx('Time: {0}m {1}s', String(Math.floor(ms / 60000)), String(Math.floor(ms / 1000) % 60)), 'white')
    this.ui.line(tx('Log: {0}', store.get().info.logFile), 'darkGray')
    await this.loadHistory()
    await this.afterClean()
    this.ui.status(tx('Cleaning finished. Click "Scan again" to see the current state.'))
    return true
  }
}
