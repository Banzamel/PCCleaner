import { useUi } from '../../../app/AppContext'
import { useT } from '../../../shared/i18n/useT'
import { Card } from '../../../shared/ui/Card'

/** The last five cleanups (history.json), newest first. */
export function HistoryCard() {
  const tr = useT()
  const history = useUi((s) => s.history)
  const list = history.slice(0, 5)
  return (
    <Card>
      <h2>{tr.t('Previous cleanups')}</h2>
      {!list.length ? (
        <p className="dim small">{tr.t('Previous cleanups will appear here.')}</p>
      ) : (
        list.map((h) => (
          <p key={h.date + h.total} className="dim small">
            {tr.t('{0}   {1} freed   (free {2} → {3})', h.date, tr.size(h.total), tr.size(h.freeBefore), tr.size(h.freeAfter))}
          </p>
        ))
      )}
    </Card>
  )
}
