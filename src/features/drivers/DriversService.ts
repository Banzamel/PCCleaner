import type { DriverRow } from '../../app/state'
import type { UiActions } from '../../app/UiActions'
import type { CommandError } from '../../core/model/types'
import { tx } from '../../core/text/format'

/** A driver can be ticked until it is installed (a failure can be retried). */
export function selectable(d: DriverRow, preview: boolean): boolean {
  return !preview && (d.state === 'new' || d.state === 'fail')
}

/** Newer drivers from Windows Update; contacted only after "Check". */
export class DriversService {
  constructor(private readonly ui: UiActions) {}

  async check(): Promise<void> {
    const { store, api } = this.ui
    await this.ui.busy(async () => {
      this.ui.status(tx('Looking for newer drivers on Windows Update (usually 1-3 minutes)...'))
      store.set({ drivers: null, problems: [], driversNote: tx('Searching Windows Update... It usually takes 1-3 minutes.') })
      void api.setProgress('indeterminate')
      try {
        const r = await api.driversCheck()
        const drivers = r.drivers.map((d) => ({ ...d, selected: true, state: 'new', error: '' }))
        store.set({ drivers, problems: r.problems, driversNote: null })
        this.ui.line(tx('Drivers: {0} newer, {1} devices with a problem', String(drivers.length), String(r.problems.length)), 'cyan')
        this.ui.status(
          drivers.length
            ? tx('Newer drivers found: {0}. Review the list and click "Install selected...".', String(drivers.length))
            : tx('Windows Update has no newer drivers for this PC.'),
        )
      } catch (e) {
        store.set({ driversNote: (e as CommandError).message })
        this.ui.status(tx('Checking for drivers failed.'))
      } finally {
        void api.setProgress('none')
      }
    })
  }

  toggle(id: string): void {
    const s = this.ui.store.get()
    if (s.busy) return
    this.ui.store.set({ drivers: (s.drivers ?? []).map((d) => (d.id === id && selectable(d, s.info.preview) ? { ...d, selected: !d.selected } : d)) })
  }

  /** Called from the driver events while installing. */
  onState(id: string, state: string, error: string): void {
    this.ui.store.set((s) => ({ drivers: (s.drivers ?? []).map((d) => (d.id === id ? { ...d, state, error } : d)) }))
  }

  async install(): Promise<void> {
    const { store, api, tr } = this.ui
    const s = store.get()
    const chosen = (s.drivers ?? []).filter((d) => d.selected && selectable(d, s.info.preview))
    if (!chosen.length) return
    const yes = await this.ui.confirm({
      title: tr.t('PC Cleaner - install drivers'),
      lines: chosen.map((d) => ({ text: `${d.name}  ${d.version}` })),
      question: tr.t('A System Restore point is created first. With a graphics driver the screen may go blank for a moment. Do not turn off the computer while installing.'),
      tone: 'info',
    })
    if (!yes) return
    await this.ui.busy(async () => {
      store.set({ installingDrivers: true })
      for (const d of chosen) this.onState(d.id, 'waiting', '')
      this.ui.status(tx('Installing drivers from Windows Update. Do not turn off the computer...'))
      void api.setProgress('indeterminate')
      try {
        const restore = await api.driversInstall(chosen.map((d) => d.id))
        this.ui.line(tx('Restore point: {0}', { text: restore }), 'green')
      } finally {
        store.set({ installingDrivers: false })
        void api.setProgress('none')
      }
      const after = (store.get().drivers ?? []).filter((d) => chosen.some((c) => c.id === d.id))
      store.set((st) => ({ drivers: (st.drivers ?? []).map((d) => (d.state === 'ok' || d.state === 'restart' ? { ...d, selected: false } : d)) }))
      const ok = after.filter((d) => d.state === 'ok').length
      const restart = after.filter((d) => d.state === 'restart').length
      const failed = after.length - ok - restart
      let t = tr.t('Drivers installed: {0} of {1}.', String(ok + restart), String(after.length))
      if (restart) t += ` ${tr.t('Restart the computer to finish installing ({0}).', String(restart))}`
      if (failed) t += ` ${tr.t('Not all of them worked - see the list and the Log.')}`
      this.ui.status({ key: t })
      await this.ui.message(t, failed ? 'warning' : 'info')
    })
  }
}
