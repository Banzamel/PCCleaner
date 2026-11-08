import type { CommandError, Text, Tone } from '../core/model/types'
import type { Commands } from '../platform/commands'
import type { Store } from '../shared/lib/Store'
import type { Dialog, PageId, UiState } from './state'

/** Window-wide actions every feature service uses: busy state, status line, log, dialogs, pages. */
export class UiActions {
  private answer: ((yes: boolean) => void) | null = null

  constructor(
    readonly store: Store<UiState>,
    readonly api: Commands,
  ) {}

  get tr() {
    return this.store.get().tr
  }

  status(text: Text): void {
    this.store.set({ status: text })
  }

  line(text: Text, tone: Tone = 'gray'): void {
    this.store.set((s) => ({ log: [...s.log.slice(-4999), { text, tone }] }))
  }

  /** Runs an action with the buttons locked; an error is shown in a message box. */
  async busy<T>(fn: () => Promise<T>): Promise<T | undefined> {
    this.store.set({ busy: true })
    try {
      return await fn()
    } catch (e) {
      const err = e as CommandError
      const text = this.tr.text(err.message ?? { key: 'An error occurred: {0}', args: [String(e)] })
      await this.message(text, 'error')
      return undefined
    } finally {
      this.store.set({ busy: false })
    }
  }

  go(page: PageId): void {
    const s = this.store.get()
    if (s.cleaning && page !== 'cleanup') return
    this.store.set({ page })
    void this.api.savePage(page)
  }

  message(text: string, tone: Dialog['tone'] = 'info'): Promise<void> {
    return this.ask({ title: 'PC Cleaner', lines: [{ text }], kind: 'message', tone }).then(() => undefined)
  }

  confirm(dialog: Omit<Dialog, 'kind'>): Promise<boolean> {
    return this.ask({ ...dialog, kind: 'confirm' })
  }

  private ask(dialog: Dialog): Promise<boolean> {
    this.answer?.(false)
    this.store.set({ dialog })
    return new Promise((resolve) => {
      this.answer = resolve
    })
  }

  /** Called by the dialog buttons. */
  close(yes: boolean): void {
    this.store.set({ dialog: null })
    const a = this.answer
    this.answer = null
    a?.(yes)
  }
}
