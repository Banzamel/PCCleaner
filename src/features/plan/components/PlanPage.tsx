import { useServices, useUi } from '../../../app/AppContext'
import { tabCount } from '../../../core/plan/listing'
import { useT } from '../../../shared/i18n/useT'
import { Card } from '../../../shared/ui/Card'
import { PageHeader } from '../../../shared/ui/PageHeader'
import { Segment } from '../../../shared/ui/Segment'
import { PlanFooter } from './PlanFooter'
import { PlanList } from './PlanList'
import { PlanToolbar } from './PlanToolbar'

export function PlanPage() {
  const tr = useT()
  const { plan } = useServices()
  const report = useUi((s) => s.report)
  const tab = useUi((s) => s.plan.tab)
  const count = (t: 'plan' | 'large') => (report ? tabCount(report.categories, report.leftovers, t) : 0)
  return (
    <div className="plan-page">
      <PageHeader
        title={tr.t('Cleanup plan')}
        info={tr.t(
          'Click a group header to expand it. The box on the left ticks an item, clicking its name shows the full description and a column header sorts the list. Files and folders go to the Recycle Bin; registry entries and tasks get a backup and a restore point.',
        )}
      />
      <div className="tab-bar">
        <Segment
          options={[
            { id: 'plan', label: tr.t('Plan ({0})', String(count('plan'))) },
            { id: 'large', label: tr.t('Large files and folders ({0})', String(count('large'))) },
          ]}
          value={tab}
          onChange={(t) => plan.setView({ tab: t })}
        />
        <span className="dim small">{tr.t('Large items are never ticked in advance.')}</span>
      </div>
      <PlanToolbar />
      <Card className="list-card">
        <PlanList />
      </Card>
      <PlanFooter />
    </div>
  )
}
