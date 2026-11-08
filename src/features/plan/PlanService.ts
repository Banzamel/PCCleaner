import type { UiActions } from '../../app/UiActions'
import type { PlanFilter, PlanTab, PlanView } from '../../core/plan/listing'
import { recommended } from '../../core/plan/rules'

/** Ticking items, the recommended plan, and the plan page view (tab, filter, search, sort, groups). */
export class PlanService {
  constructor(
    private readonly ui: UiActions,
    private readonly onNewData: () => void,
  ) {}

  private canChange(): boolean {
    const s = this.ui.store.get()
    return !!s.report && !s.busy && !s.cleaned
  }

  toggle(id: string): void {
    if (!this.canChange()) return
    const s = this.ui.store.get()
    const cat = s.report?.categories.find((c) => c.id === id)
    // A blocked category (browser running) cannot be ticked; the status line says why
    if (cat?.blocker && !s.selected.has(id)) {
      this.ui.status(cat.blocker)
      return
    }
    const selected = new Set(s.selected)
    if (selected.has(id)) selected.delete(id)
    else selected.add(id)
    this.ui.store.set({ selected })
    this.onNewData()
  }

  selectRecommended(): void {
    const r = this.ui.store.get().report
    if (!r || !this.canChange()) return
    this.ui.store.set({ selected: recommended(r.categories, r.leftovers) })
    this.onNewData()
  }

  clearSelection(): void {
    if (!this.canChange()) return
    this.ui.store.set({ selected: new Set() })
    this.onNewData()
  }

  setView(patch: Partial<PlanView>): void {
    this.ui.store.set((s) => ({ plan: { ...s.plan, ...patch } }))
  }

  /** A column header: the same column flips the order, another one sorts by it. */
  sortBy(key: PlanView['sort']['key']): void {
    const cur = this.ui.store.get().plan.sort
    this.setView({ sort: cur.key === key ? { key, desc: !cur.desc } : { key, desc: key !== 'name' } })
  }

  toggleGroup(key: string): void {
    const expanded = new Set(this.ui.store.get().expanded)
    if (expanded.has(key)) expanded.delete(key)
    else expanded.add(key)
    this.ui.store.set({ expanded })
  }

  /** A tile on the overview: the plan on the matching tab and filter, all groups open. */
  openTile(tab: PlanTab, filter: PlanFilter): void {
    if (!this.ui.store.get().report) return
    this.setView({ tab, filter })
    this.ui.store.set({ expanded: new Set(['categories', 'leftovers', 'registry', 'largefolders', 'largefiles']) })
    this.ui.go('plan')
  }
}
