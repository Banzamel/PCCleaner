import { useServices, useUi } from '../../../app/AppContext'
import { useT } from '../../../shared/i18n/useT'
import { Button } from '../../../shared/ui/Button'
import { PageHeader } from '../../../shared/ui/PageHeader'
import { DriveList } from './DriveList'
import { ResultCard } from './ResultCard'
import { SpaceBars } from './SpaceBars'
import { StartupCard } from './StartupCard'

export function OverviewPage() {
  const tr = useT()
  const { scan } = useServices()
  const busy = useUi((s) => s.busy)
  const scanned = useUi((s) => s.report !== null)
  const lastScan = useUi((s) => s.lastScan)
  const drives = useUi((s) => s.ticked)
  const status = lastScan ? tr.t('Last scan: today, {0} · drives {1} · nothing has been deleted yet', lastScan, drives.join(', ')) : ''
  return (
    <div className="overview">
      <PageHeader
        title={tr.t('Overview')}
        info={tr.t('Pick the drives and click "Scan". PC Cleaner checks system junk, app caches and leftovers of uninstalled programs and games.')}
        status={status}
      >
        <Button look="outline" disabled={busy} onClick={() => void scan.scan()}>
          {scanned ? tr.t('Scan again') : tr.t('Scan')}
        </Button>
      </PageHeader>
      <ResultCard />
      <div className="overview-bottom">
        <SpaceBars />
        <div className="overview-right">
          <DriveList />
          <StartupCard />
        </div>
      </div>
    </div>
  )
}
