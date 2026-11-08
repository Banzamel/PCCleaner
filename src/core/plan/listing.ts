import type { Category, Leftover, Text } from '../model/types'
import { tx } from '../text/format'
import { goesToRecycleBin, isLarge } from './rules'

export type PlanTab = 'plan' | 'large'
export type PlanFilter = 'all' | 'selected' | 'optin'
export type SortKey = 'name' | 'size' | 'confidence'

export interface PlanView {
  tab: PlanTab
  filter: PlanFilter
  search: string
  sort: { key: SortKey; desc: boolean }
}

export type PlanItem = { cat: true; c: Category } | { cat: false; l: Leftover }

export interface PlanGroup {
  key: string
  title: Text
  items: PlanItem[]
  /** All items of the group, for the "selected" total even when filtered. */
  all: PlanItem[]
}

export const itemId = (i: PlanItem): string => (i.cat ? i.c.id : i.l.id)
export const itemBytes = (i: PlanItem): number => (i.cat ? i.c.bytes : i.l.bytes)

/** Text searched by the search box: name, description and notes, or path, reason and AI text. */
export function matches(i: PlanItem, view: PlanView, selected: Set<string>, render: (t: Text) => string, ai: Record<string, string>): boolean {
  if (view.filter === 'selected' && !selected.has(itemId(i))) return false
  if (view.filter === 'optin' && !(i.cat ? i.c.optIn : i.l.confidence === 'low')) return false
  const q = view.search.trim().toLowerCase()
  if (!q) return true
  const t = i.cat
    ? [render(i.c.name), render(i.c.description), i.c.note ? render(i.c.note) : ''].join(' ')
    : [i.l.path, i.l.name, render(i.l.reason), ai[i.l.id] ?? ''].join(' ')
  return t.toLowerCase().includes(q)
}

const RANK: Record<string, number> = { high: 3, medium: 2, low: 1 }

export function sortItems(items: PlanItem[], sort: PlanView['sort'], name: (i: PlanItem) => string): PlanItem[] {
  const key = (i: PlanItem): number | string => {
    if (sort.key === 'name') return name(i).toLowerCase()
    if (sort.key === 'confidence') return i.cat ? (i.c.optIn ? 0 : 4) : RANK[i.l.confidence] ?? 0
    return itemBytes(i)
  }
  return [...items].sort((a, b) => {
    const ka = key(a)
    const kb = key(b)
    const c = ka < kb ? -1 : ka > kb ? 1 : 0
    return (sort.desc ? -c : c) || itemBytes(b) - itemBytes(a)
  })
}

/** The groups of the plan page: categories, recycle-bin leftovers, registry/tasks; or the large items. */
export function groups(categories: Category[], leftovers: Leftover[], tab: PlanTab): PlanGroup[] {
  const L = (l: Leftover): PlanItem => ({ cat: false, l })
  const make = (key: string, title: Text, all: PlanItem[]): PlanGroup => ({ key, title, items: all, all })
  if (tab === 'large') {
    const large = leftovers.filter(isLarge)
    return [
      make('largefolders', tx('Large folders (over 1 GB) - often live games and programs, check each one'), large.filter((l) => l.kind === 'folder').map(L)),
      make('largefiles', tx('Large files in the profile and in Downloads'), large.filter((l) => l.kind === 'file').map(L)),
    ]
  }
  const cats = categories.filter((c) => c.bytes > 0 || ['dns', 'dism', 'docker', 'dockerall'].includes(c.kind))
  const left = leftovers.filter((l) => !isLarge(l))
  return [
    make('categories', tx('Cleanup categories - they rebuild themselves'), cats.map((c) => ({ cat: true, c }))),
    make('leftovers', tx('Leftovers of uninstalled programs and games'), left.filter(goesToRecycleBin).map(L)),
    make('registry', tx('Registry and tasks'), left.filter((l) => !goesToRecycleBin(l)).map(L)),
  ]
}

/** Number on the tab: non-empty categories plus leftovers (or the large items). */
export function tabCount(categories: Category[], leftovers: Leftover[], tab: PlanTab): number {
  if (tab === 'large') return leftovers.filter(isLarge).length
  return categories.filter((c) => c.bytes > 0).length + leftovers.filter((l) => !isLarge(l)).length
}

/** Group header total: "X selected", "selected: N" or "nothing selected". */
export function groupTotal(all: PlanItem[], selected: Set<string>): Text {
  const sel = all.filter((i) => selected.has(itemId(i)))
  if (!sel.length) return tx('nothing selected')
  const b = sel.reduce((s, i) => s + itemBytes(i), 0)
  return b > 0 ? tx('{0} selected', { size: b }) : tx('selected: {0}', String(sel.length))
}
