import type { UiActions } from '../../app/UiActions'
import type { StartupEntry } from '../../core/model/types'
import { tx } from '../../core/text/format'
import type { AssistantService } from '../assistant'

/** Startup apps: the switch works like Task Manager; the bin removes an entry with a backup. */
export class StartupService {
  constructor(
    private readonly ui: UiActions,
    private readonly assistant: AssistantService,
  ) {}

  async load(): Promise<void> {
    const list = await this.ui.api.startupList().catch(() => null)
    if (list) this.ui.store.set({ startup: list })
    this.assistant.reset()
  }

  async refresh(): Promise<void> {
    await this.ui.busy(async () => {
      this.ui.status(tx('Reading the programs that start at sign-in...'))
      await this.load()
      this.ui.status(tx('Startup apps loaded. The switch on the right turns a program on or off.'))
    })
  }

  private replace(e: StartupEntry, patch: Partial<StartupEntry> | null): void {
    this.ui.store.set((s) => ({
      startup: (s.startup ?? []).flatMap((x) => (x.id !== e.id ? [x] : patch ? [{ ...x, ...patch }] : [])),
    }))
    this.assistant.reset()
  }

  /** Applies at once, like in Task Manager (turning off deletes nothing). */
  async toggle(e: StartupEntry): Promise<void> {
    const s = this.ui.store.get()
    if (s.busy) return
    if (s.info.preview) return this.ui.status(tx('Preview mode - startup apps are not changed.'))
    if (e.protected) return this.ui.status(tx('{0}: protected - {1}', e.name, { text: e.protected }))
    const enable = !e.isEnabled
    try {
      await this.ui.api.startupSet(e.id, enable)
      this.replace(e, { isEnabled: enable })
      const state = tx(enable ? 'on' : 'off')
      this.ui.line(tx('Startup: {0} -> {1}', e.name, { text: state }), 'green')
      this.ui.status(tx('{0}: startup {1}. It takes effect at the next sign-in.', e.name, { text: state }))
    } catch (err) {
      const m = (err as { detail?: string }).detail ?? String(err)
      await this.ui.message(this.ui.tr.t('Could not change {0}: {1}', e.name, m), 'warning')
    }
  }

  /** Permanent removal after a confirmation; a .reg / .xml backup, or the shortcut goes to the Recycle Bin. */
  async remove(e: StartupEntry): Promise<void> {
    const s = this.ui.store.get()
    const tr = this.ui.tr
    if (s.busy || s.info.preview || e.protected) return
    const note =
      e.kind === 'folder'
        ? tr.t('The shortcut goes to the Recycle Bin.')
        : e.kind === 'task'
          ? tr.t('The task gets an .xml backup (Task Scheduler > Import Task restores it).')
          : tr.t('The entry gets a .reg backup (double-clicking it restores the entry).')
    const yes = await this.ui.confirm({
      title: tr.t('PC Cleaner - remove from startup'),
      lines: [{ text: e.name }, { text: e.command, tone: 'darkGray' }, { text: tr.t('The program is not uninstalled, only its startup entry is removed. {0}', note) }],
      question: tr.t('Remove from startup permanently?'),
      tone: 'warning',
    })
    if (!yes) return
    try {
      const place = await this.ui.api.startupRemove(e.id)
      this.replace(e, null)
      this.ui.line(tx('Startup: {0} removed (backup: {1})', e.name, { text: place }), 'green')
      this.ui.status(tx('{0}: removed from startup. Backup: {1}', e.name, { text: place }))
    } catch (err) {
      const m = (err as { detail?: string }).detail ?? String(err)
      await this.ui.message(tr.t('Could not remove {0}: {1}', e.name, m), 'warning')
    }
  }

  async describe(): Promise<void> {
    await this.ui.busy(async () => {
      this.ui.status(tx('The model is describing the programs (about 20-40 s)...'))
      const n = await this.assistant.describeStartup(this.ui.store.get().startup ?? [])
      this.ui.status(n >= 0 ? tx('{0} items described.', String(n)) : tx('The assistant is unavailable - see the Log.'))
    })
  }
}
