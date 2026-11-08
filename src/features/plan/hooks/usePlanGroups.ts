import { useMemo } from 'react'

import { useUi } from '../../../app/AppContext'
import { leftoverLabel } from '../../../core/plan/describe'
import { groups, matches, sortItems, type PlanGroup } from '../../../core/plan/listing'
import { useT } from '../../../shared/i18n/useT'

/** The plan groups for the current tab, filtered, searched and sorted. Empty groups are dropped. */
export function usePlanGroups(): PlanGroup[] {
  const tr = useT()
  const report = useUi((s) => s.report)
  const view = useUi((s) => s.plan)
  const selected = useUi((s) => s.selected)
  const ai = useUi((s) => s.ai)
  return useMemo(() => {
    if (!report) return []
    const name = (i: Parameters<typeof matches>[0]) => (i.cat ? tr.text(i.c.name) : tr.text(leftoverLabel(i.l)))
    return groups(report.categories, report.leftovers, view.tab)
      .map((g) => ({ ...g, items: sortItems(g.all.filter((i) => matches(i, view, selected, tr.text, ai)), view.sort, name) }))
      .filter((g) => g.items.length > 0)
  }, [report, view, selected, ai, tr])
}
