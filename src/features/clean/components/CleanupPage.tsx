import { useT } from '../../../shared/i18n/useT'
import { Card } from '../../../shared/ui/Card'
import { PageHeader } from '../../../shared/ui/PageHeader'
import { CleanResultCard } from './CleanResultCard'
import { HistoryCard } from './HistoryCard'
import { ProgressCard } from './ProgressCard'
import { StepList } from './StepList'

/** The progress screen: steps on the left, the result and previous cleanups on the right. */
export function CleanupPage() {
  const tr = useT()
  return (
    <div className="cleanup-page">
      <div className="cleanup-left">
        <PageHeader
          title={tr.t('Cleaning')}
          info={tr.t('Do not turn off the computer. Files in use are skipped; folders and shortcuts go to the Recycle Bin.')}
        />
        <ProgressCard />
        <Card className="list-card">
          <StepList />
        </Card>
      </div>
      <aside className="cleanup-aside">
        <CleanResultCard />
        <HistoryCard />
        <Card className="note-card">
          <p className="dim small">{tr.t('Progress is also shown on the PC Cleaner taskbar icon, so the window can work in the background.')}</p>
        </Card>
      </aside>
    </div>
  )
}
