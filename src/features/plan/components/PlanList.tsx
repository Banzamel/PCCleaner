import { useState } from 'react'

import { useServices, useUi } from '../../../app/AppContext'
import { groupTotal, itemId, type SortKey } from '../../../core/plan/listing'
import { useT } from '../../../shared/i18n/useT'
import { EmptyText } from '../../../shared/ui/EmptyText'
import { usePlanGroups } from '../hooks/usePlanGroups'
import { PlanRow } from './PlanRow'

/** Column headers (click sorts), collapsible groups and the rows. */
export function PlanList() {
  const tr = useT()
  const { plan } = useServices()
  const groups = usePlanGroups()
  const report = useUi((s) => s.report)
  const view = useUi((s) => s.plan)
  const expanded = useUi((s) => s.expanded)
  const selected = useUi((s) => s.selected)
  const [open, setOpen] = useState<string | null>(null)
  const columns: Array<{ key: SortKey; label: string }> = [
    { key: 'name', label: tr.t('ITEM') },
    { key: 'size', label: tr.t('SIZE') },
    { key: 'confidence', label: tr.t('CONFIDENCE') },
  ]
  const empty = !report
    ? tr.t('The plan appears after a scan. Go back to Overview and click "Scan".')
    : view.tab === 'large' && !groups.length
      ? tr.t('The scan found no large files or folders.')
      : tr.t('Nothing matches the search or filter.')
  return (
    <div className="plan-list">
      <div className="plan-columns" role="row">
        {columns.map((c) => (
          <button key={c.key} type="button" className={`col-${c.key} ${view.sort.key === c.key ? 'on' : ''}`} onClick={() => plan.sortBy(c.key)}>
            {c.label}
            {view.sort.key === c.key ? (view.sort.desc ? ' ▼' : ' ▲') : ''}
          </button>
        ))}
      </div>
      <div className="rows">
        {!groups.length ? <EmptyText>{empty}</EmptyText> : null}
        {groups.map((g) => {
          // Groups start collapsed (header and total only); a search shows everything
          const collapsed = !expanded.has(g.key) && !view.search.trim()
          return (
            <section key={g.key}>
              <button type="button" className="group-header" aria-expanded={!collapsed} onClick={() => plan.toggleGroup(g.key)}>
                <span>
                  {collapsed ? '▸' : '▾'} {tr.text(g.title)} ({g.items.length})
                </span>
                <span className="dim small">{tr.text(groupTotal(g.all, selected))}</span>
              </button>
              {collapsed
                ? null
                : g.items.map((i) => (
                    <PlanRow key={itemId(i)} item={i} open={open === itemId(i)} onOpen={() => setOpen(open === itemId(i) ? null : itemId(i))} />
                  ))}
            </section>
          )
        })}
      </div>
    </div>
  )
}
