import { tx } from '../core/text/format'
import { findUpdate, type AvailableUpdate } from '../platform/desktop'
import type { UiActions } from './UiActions'

const RELEASES = 'https://github.com/Banzamel/PCCleaner/releases/latest'

/** One request to GitHub Releases at start (no data about the computer); off with --no-update-check. */
export class UpdateService {
  private found: AvailableUpdate | null = null

  constructor(private readonly ui: UiActions) {}

  async check(): Promise<void> {
    this.ui.status(tx('Checking for updates...'))
    try {
      this.found = await findUpdate()
      if (this.found) this.ui.store.set({ update: { version: this.found.version } })
    } catch (e) {
      this.ui.line(tx('Update check: {0}', String(e)), 'darkGray')
    }
    this.ui.status(tx('Ready. Pick the drives and click "Scan" (F5).'))
  }

  /** Downloads and runs the signed installer; the program closes. The portable version opens the page. */
  async install(): Promise<void> {
    const s = this.ui.store.get()
    if (s.info.portable || s.info.otherUser || !this.found) {
      this.ui.line(tx('Portable version - download the new version from: {0}', RELEASES), 'yellow')
      await this.ui.api.openTarget('url', RELEASES)
      return
    }
    const update = this.found
    await this.ui.busy(async () => {
      this.ui.status(tx('Downloading {0}...', `PC Cleaner ${update.version}`))
      await update.install()
    })
  }
}
