import { AnalysisPage } from '../../features/analysis'
import { ChatPage } from '../../features/assistant'
import { CleanupPage } from '../../features/clean'
import { DriversPage } from '../../features/drivers'
import { LogPage } from '../../features/log'
import { OverviewPage } from '../../features/overview'
import { PlanPage } from '../../features/plan'
import { SettingsPage } from '../../features/settings'
import { StartupPage } from '../../features/startup'
import { useUi } from '../AppContext'
import { useShortcuts } from '../useShortcuts'
import { DialogHost } from './DialogHost'
import { NavBar } from './NavBar'
import { StatusBar } from './StatusBar'

const PAGES = {
  overview: OverviewPage,
  plan: PlanPage,
  analysis: AnalysisPage,
  startup: StartupPage,
  drivers: DriversPage,
  chat: ChatPage,
  log: LogPage,
  settings: SettingsPage,
  cleanup: CleanupPage,
}

export function App() {
  const page = useUi((s) => s.page)
  useShortcuts()
  const Page = PAGES[page]
  return (
    <div className="shell">
      <NavBar />
      <div className="content">
        <main className="page">
          <Page />
        </main>
        <StatusBar />
      </div>
      <DialogHost />
    </div>
  )
}
