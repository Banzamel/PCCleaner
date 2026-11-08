import { recommended } from '../core/plan/rules'
import { selectedTotal } from '../core/plan/rules'
import type { CleanService } from '../features/clean'
import type { ScanService } from '../features/overview'
import type { UiActions } from './UiActions'

/**
 * Automatic cleaning at sign-in (--auto-clean from the Task Scheduler task): scan, clean only the
 * categories the plan recommends (never leftovers or large files), show the result in a notification.
 * The program closes itself after 15 s unless the user opened the window.
 */
export class AutoCleanService {
  constructor(
    private readonly ui: UiActions,
    private readonly scan: ScanService,
    private readonly clean: CleanService,
  ) {}

  async run(): Promise<void> {
    const { api, store, tr } = this.ui
    await api.autoCleanDone().catch(() => undefined)
    await api.trayTexts(tr.t('Open PC Cleaner'), tr.t('Exit'), tr.t('PC Cleaner - automatic cleaning...'))
    await this.scan.scan()
    const r = store.get().report
    let msg: string
    if (!r) {
      msg = tr.t('The cleaning failed - see the Log.')
    } else {
      const cats = r.categories.filter((c) => recommended(r.categories, []).has(c.id)).map((c) => c.id)
      store.set({ selected: new Set(cats) })
      const bytes = selectedTotal(r.categories, r.leftovers, store.get().selected)
      if (store.get().info.preview) msg = tr.t('Preview mode: {0} to clean, nothing was deleted.', tr.size(bytes))
      else if (bytes <= 0) msg = tr.t('There was nothing to clean.')
      else {
        const ok = await this.clean.run()
        const result = store.get().cleanResult
        msg = ok && result ? tr.t('{0} freed. Click to see the details.', tr.size(result.total)) : tr.t('The cleaning failed - see the Log.')
      }
    }
    await api.trayTexts(tr.t('Open PC Cleaner'), tr.t('Exit'), 'PC Cleaner')
    await api.notify(tr.t('PC Cleaner - automatic cleaning'), msg)
    await new Promise((res) => setTimeout(res, 15000))
    if (!(await api.windowVisible())) await api.exitApp()
  }
}
