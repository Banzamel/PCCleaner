import type { Category, Leftover, Text } from '../model/types'
import { tx } from '../text/format'
import type { PlanFilter, PlanTab } from '../plan/listing'
import { goesToRecycleBin, isLarge, selectedTotal } from '../plan/rules'

export interface Tile {
  title: Text
  bytes: number
  count: number
  tone: 'green' | 'warning' | 'accent'
  tab: PlanTab
  filter: PlanFilter
}

/** Three tiles next to the result: safe to delete, to review, large files. Each opens the plan. */
export function tiles(categories: Category[], leftovers: Leftover[], selected: Set<string>): Tile[] {
  const large = leftovers.filter(isLarge)
  const review = [...categories, ...leftovers.filter((l) => !isLarge(l))].filter((x) => !selected.has(x.id) && x.bytes > 0)
  const sum = (xs: Array<{ bytes: number }>) => xs.reduce((s, x) => s + x.bytes, 0)
  const sel = [...categories, ...leftovers].filter((x) => selected.has(x.id))
  return [
    { title: tx('Safe to delete'), bytes: selectedTotal(categories, leftovers, selected), count: sel.length, tone: 'green', tab: 'plan', filter: 'selected' },
    { title: tx('To review'), bytes: sum(review), count: review.length, tone: 'warning', tab: 'plan', filter: 'all' },
    { title: tx('Large files and folders'), bytes: sum(large), count: large.length, tone: 'accent', tab: 'large', filter: 'all' },
  ]
}

export interface Bar {
  name: Text
  bytes: number
  selected: boolean
}

/** Bars: the largest categories, the leftovers combined, the rest combined (at most 7 bars). */
export function bars(categories: Category[], leftovers: Leftover[], selected: Set<string>): Bar[] {
  let list: Bar[] = categories.filter((c) => c.bytes > 0).map((c) => ({ name: c.name, bytes: c.bytes, selected: selected.has(c.id) }))
  const bin = leftovers.filter(goesToRecycleBin)
  if (bin.length) {
    list.push({ name: tx('Program leftovers'), bytes: bin.reduce((s, l) => s + l.bytes, 0), selected: bin.some((l) => selected.has(l.id)) })
  }
  list.sort((a, b) => b.bytes - a.bytes)
  if (list.length > 7) {
    const rest = list.slice(6)
    list = [...list.slice(0, 6), { name: tx('Other items'), bytes: rest.reduce((s, b) => s + b.bytes, 0), selected: rest.some((b) => b.selected) }]
  }
  return list
}
