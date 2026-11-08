import { Translator, type Dictionary } from '../core/text/Translator'
import { tx } from '../core/text/format'
import es from '../lang/es.json'
import pl from '../lang/pl.json'
import { commands } from '../platform/commands'
import { setWindowTheme } from '../platform/desktop'
import { on } from '../platform/events'
import { Store } from '../shared/lib/Store'
import { createServices, type Services } from './services'
import type { PageId, UiState } from './state'
import { UiActions } from './UiActions'

const DICTIONARIES: Record<string, Dictionary> = { pl, es }
const START_PAGES: PageId[] = ['analysis', 'startup', 'drivers', 'chat', 'log', 'settings']

/** Applies the theme: the Settings choice, or the Windows app mode. */
export function applyTheme(theme: string): void {
  const t = theme === 'light' || theme === 'dark' ? theme : null
  document.documentElement.dataset.theme = t ?? (matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light')
  void setWindowTheme(t)
}

/** Loads how the program was started, builds the store and the services and wires the events. */
export async function boot(): Promise<{ store: Store<UiState>; services: Services }> {
  const [info, config] = await Promise.all([commands.appInfo(), commands.getConfig()])
  const tr = new Translator(info.language, DICTIONARIES[info.language] ?? {})
  document.documentElement.lang = info.language
  applyTheme(info.theme)
  const store = new Store<UiState>({
    info, tr, config,
    page: START_PAGES.includes(info.page as PageId) ? (info.page as PageId) : 'overview',
    busy: false, status: tx('Ready. Pick the drives and click "Scan" (F5).'),
    drives: [], ticked: [], report: null, selected: new Set(), scanning: false, stage: null, lastScan: null,
    cleaning: false, cleaned: false, steps: [], cleanResult: null, history: [], log: [],
    plan: { tab: 'plan', filter: 'all', search: '', sort: { key: 'size', desc: true } }, expanded: new Set(),
    analyses: {}, analysisLetter: 'C', startup: null, drivers: null, problems: [], driversNote: null, installingDrivers: false,
    ai: {}, chat: [], update: null, dialog: null,
  })
  const ui = new UiActions(store, commands)
  const services = createServices(ui)
  on('pcc://status', (t) => {
    store.set({ status: t })
    if (store.get().scanning) store.set({ stage: t })
  })
  on('pcc://line', ({ text, tone }) => {
    ui.line(text, tone)
    if (store.get().scanning) store.set({ stage: text })
  })
  on('pcc://step', (s) => services.clean.onStep(s.id, s.state, s.bytes, s.note))
  on('pcc://driver', (d) => services.drivers.onState(d.id, d.state, d.error))
  on('pcc://close-blocked', (why) =>
    void ui.message(
      tr.t(why === 'drivers' ? 'Wait for the driver installation to finish.' : 'Wait for the cleaning to finish - stopping halfway could leave services paused or File Explorer closed.'),
      'warning',
    ),
  )
  on('pcc://tray-hint', () => void commands.notify('PC Cleaner', tr.t('The program keeps running in the notification area. Click the icon to open it.')))
  matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => applyTheme(store.get().info.theme))
  void commands.trayTexts(tr.t('Open PC Cleaner'), tr.t('Exit'), 'PC Cleaner')
  await services.scan.loadDrives()
  void services.clean.loadHistory()
  return { store, services }
}

/** Work after the window is shown: the update check, or the automatic cleaning. */
export function afterStart(store: Store<UiState>, services: Services): void {
  const { info } = store.get()
  if (info.autoClean) {
    void services.autoClean.run()
    return
  }
  if (!info.noUpdateCheck) void services.update.check()
}
