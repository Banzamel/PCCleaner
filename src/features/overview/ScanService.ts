import type { UiActions } from '../../app/UiActions'
import { recommended } from '../../core/plan/rules'
import { tx } from '../../core/text/format'

/** Drives to scan and the full scan. The scan deletes nothing; it fills the plan. */
export class ScanService {
  constructor(
    private readonly ui: UiActions,
    private readonly onNewData: () => void,
    private readonly loadStartup: () => Promise<void>,
  ) {}

  /** Fixed drives; all ticked unless --drives chose some. The system drive is always scanned. */
  async loadDrives(): Promise<void> {
    const drives = await this.ui.api.listDrives()
    const wanted = this.ui.store.get().info.drives.map((d) => `${d.toUpperCase()}:`)
    const ticked = drives.filter((d) => d.system || !wanted.length || wanted.includes(d.letter)).map((d) => d.letter)
    const sys = drives.find((d) => d.system)?.letter ?? drives[0]?.letter ?? 'C:'
    this.ui.store.set({ drives, ticked, analysisLetter: sys.slice(0, 1) })
  }

  toggleDrive(letter: string): void {
    const s = this.ui.store.get()
    const d = s.drives.find((x) => x.letter === letter)
    if (!d || d.system || s.busy) return
    this.ui.store.set({ ticked: s.ticked.includes(letter) ? s.ticked.filter((x) => x !== letter) : [...s.ticked, letter] })
  }

  async scan(): Promise<void> {
    const { store, api } = this.ui
    this.ui.go('overview')
    store.set({ log: [], scanning: true, stage: null, report: null, cleaned: false, cleanResult: null })
    this.ui.status(tx('Scanning - nothing is deleted'))
    this.ui.line(tx('1/2  Scanning drives: {0}', store.get().ticked.join(', ')), 'cyan')
    void api.setProgress('indeterminate')
    try {
      const report = await this.ui.busy(() => api.scan(store.get().ticked))
      if (!report) {
        store.set({ stage: tx('See the Log for details.') })
        return
      }
      const time = new Date().toTimeString().slice(0, 5)
      store.set({ report, selected: recommended(report.categories, report.leftovers), lastScan: time, ai: {} })
      this.onNewData()
      if (store.get().startup === null) {
        this.ui.status(tx('Reading the programs that start at sign-in...'))
        await this.loadStartup()
      }
      this.ui.status(tx('Scan finished. Review the plan and click "Clean selected...".'))
    } finally {
      store.set({ scanning: false })
      void api.setProgress('none')
    }
  }
}
