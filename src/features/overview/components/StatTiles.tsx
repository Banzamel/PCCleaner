import { useServices, useUi } from '../../../app/AppContext'
import { tiles } from '../../../core/overview/overview'
import { useT } from '../../../shared/i18n/useT'

/** Safe to delete / To review / Large files and folders; a click opens the plan where it belongs. */
export function StatTiles() {
  const tr = useT()
  const { plan } = useServices()
  const report = useUi((s) => s.report)
  const selected = useUi((s) => s.selected)
  if (!report) return null
  return (
    <div className="stat-tiles">
      {tiles(report.categories, report.leftovers, selected).map((t) => (
        <button key={t.title.key} type="button" className="stat-tile" onClick={() => plan.openTile(t.tab, t.filter)}>
          <span className={`stat-mark mark-${t.tone}`} />
          <span className="stat-text">
            <span className="dim small">
              {tr.text(t.title)} · {tr.t('items: {0}', String(t.count))}
            </span>
            <strong>{tr.size(t.bytes)}</strong>
          </span>
          <span className="stat-go">{tr.t('Show ›')}</span>
        </button>
      ))}
    </div>
  )
}
