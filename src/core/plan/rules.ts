import type { Category, Leftover } from '../model/types'

/** A large item: a big file from the profile or Downloads, or a leftover folder of 1 GB or more. */
export const LARGE_ITEM = 1024 ** 3

export function isLarge(l: Leftover): boolean {
  return l.largeFile || ((l.kind === 'folder' || l.kind === 'file') && l.bytes >= LARGE_ITEM)
}

export function goesToRecycleBin(l: Leftover): boolean {
  return l.kind === 'folder' || l.kind === 'file'
}

/**
 * The recommended plan, built by rules (no model): categories that contain anything, except opt-in
 * ones, blocked ones and those without a measurable folder size; leftovers with high or medium
 * confidence that are not large (large items are more often something the user still wants).
 */
export function recommended(categories: Category[], leftovers: Leftover[]): Set<string> {
  const ids = new Set<string>()
  for (const c of categories) {
    if (!c.optIn && !c.blocker && (c.kind === 'folder' || c.kind === 'thumbnails') && c.bytes > 0) ids.add(c.id)
  }
  for (const l of leftovers) {
    if ((l.confidence === 'high' || l.confidence === 'medium') && !isLarge(l)) ids.add(l.id)
  }
  return ids
}

/** Total bytes of the selected items. */
export function selectedTotal(categories: Category[], leftovers: Leftover[], selected: Set<string>): number {
  return [...categories, ...leftovers].filter((x) => selected.has(x.id)).reduce((s, x) => s + x.bytes, 0)
}

/** Bytes locked by running programs in the selected categories. */
export function lockedTotal(categories: Category[], selected: Set<string>): number {
  return categories.filter((c) => selected.has(c.id)).reduce((s, c) => s + c.locked, 0)
}

/** A restore point is needed before changes the Recycle Bin cannot undo: registry, tasks, drivers. */
export function restorePointNeeded(categories: Category[], leftovers: Leftover[], selected: Set<string>): boolean {
  return (
    leftovers.some((l) => selected.has(l.id) && !goesToRecycleBin(l)) ||
    categories.some((c) => selected.has(c.id) && c.kind === 'driverstore')
  )
}
