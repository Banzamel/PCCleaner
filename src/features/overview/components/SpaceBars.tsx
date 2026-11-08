import { useUi } from '../../../app/AppContext'
import { bars } from '../../../core/overview/overview'
import { useT } from '../../../shared/i18n/useT'
import { Bar } from '../../../shared/ui/Bar'
import { Card } from '../../../shared/ui/Card'

/** What takes up the space you can free: the largest plan items as bars, grey = not selected. */
export function SpaceBars() {
  const tr = useT()
  const report = useUi((s) => s.report)
  const selected = useUi((s) => s.selected)
  const list = report ? bars(report.categories, report.leftovers, selected) : []
  const max = Math.max(1, ...list.map((b) => b.bytes))
  return (
    <Card className="space-bars">
      <div className="card-title-row">
        <h2>{tr.t('What takes up space you can free')}</h2>
        {list.length ? <span className="dim small">{tr.t('grey bar = not selected in the plan')}</span> : null}
      </div>
      {!list.length ? (
        <p className="dim">
          {report ? tr.t('The scan found nothing to free.') : tr.t('After the scan you will see the largest cleanup plan items here, largest first.')}
        </p>
      ) : (
        <ul className="bars-list">
          {list.map((b) => (
            <li key={b.name.key + b.bytes}>
              <span className="bars-name">{tr.text(b.name)}</span>
              <Bar value={b.bytes / max} tone={b.selected ? 'accent' : 'grey'} size="thick" />
              <strong>{tr.size(b.bytes)}</strong>
            </li>
          ))}
        </ul>
      )}
    </Card>
  )
}
