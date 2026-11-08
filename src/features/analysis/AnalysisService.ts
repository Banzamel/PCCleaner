import type { UiActions } from '../../app/UiActions'
import { tx } from '../../core/text/format'
import type { AssistantService } from '../assistant'

/** "What takes up space": a read-only drive analysis and its AI descriptions. */
export class AnalysisService {
  constructor(
    private readonly ui: UiActions,
    private readonly assistant: AssistantService,
  ) {}

  pick(letter: string): void {
    this.ui.store.set({ analysisLetter: letter })
  }

  async analyse(): Promise<void> {
    const letter = this.ui.store.get().analysisLetter
    await this.ui.busy(async () => {
      this.ui.line(tx('Scanning drive {0}: (usually 30 s - 3 min)...', letter), 'darkGray')
      const a = await this.ui.api.analyse(letter)
      this.ui.store.set((s) => ({ analyses: { ...s.analyses, [letter]: a } }))
      this.assistant.reset()
      this.ui.status(tx('Drive {0}: analysis ready.', letter))
    })
  }

  async describe(): Promise<void> {
    const s = this.ui.store.get()
    const a = s.analyses[s.analysisLetter]
    if (!a) return
    await this.ui.busy(async () => {
      this.ui.status(tx('The model is describing the items (about 20-40 s)...'))
      const n = await this.assistant.describeAnalysis(a)
      this.ui.status(n >= 0 ? tx('{0} items described.', String(n)) : tx('The assistant is unavailable - see the Log.'))
    })
  }
}
