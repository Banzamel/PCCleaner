import type { Category, Leftover, Text } from '../model/types'
import { tx } from '../text/format'

/** How a leftover is named in lists: tasks and registry values with their location. */
export function leftoverLabel(l: Leftover): Text {
  if (l.kind === 'task') return tx('{0}: {1}', { text: tx('Task Scheduler') }, l.path)
  const leaf = l.path.split('\\').pop()
  if ((l.kind === 'registry' || l.kind === 'regvalue') && l.name !== leaf) return tx('{0}  [{1}]', l.name, l.path)
  return tx('{0}', l.path)
}

/** Size column: registry entries and tasks have none. */
export function leftoverSize(l: Leftover): Text {
  if (l.kind === 'registry' || l.kind === 'regvalue') return tx('registry')
  if (l.kind === 'task') return tx('task')
  return tx('{0}', { size: l.bytes })
}

/** Size of a category, or "-" when it has no measurable size (DNS, DISM, Docker not running). */
export function categorySize(c: Category): Text {
  return c.measurable ? tx('{0}', { size: c.bytes }) : tx('-')
}

export function confidenceName(c: string): string {
  return { high: 'high', medium: 'medium', low: 'low' }[c] ?? c
}
