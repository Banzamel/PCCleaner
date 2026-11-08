import type { Category, Leftover, Text, Tone } from '../model/types'
import { tx } from '../text/format'
import { categorySize, leftoverLabel, leftoverSize } from './describe'
import { goesToRecycleBin, lockedTotal } from './rules'

export interface SummaryLine {
  text: Text
  tone: Tone
  /** Indented list entry with a value on the right. */
  value?: Text
}

const line = (text: Text, tone: Tone = 'gray', value?: Text): SummaryLine => ({ text, tone, value })

/** The summary shown before cleaning, or null when nothing is selected. */
export function summary(categories: Category[], leftovers: Leftover[], selected: Set<string>, freeBefore: number, sys: string): SummaryLine[] | null {
  const cats = categories.filter((c) => selected.has(c.id))
  const lefts = leftovers.filter((l) => selected.has(l.id))
  if (!cats.length && !lefts.length) return null
  const out: SummaryLine[] = []
  if (cats.length) {
    out.push(line(tx('To be cleaned:'), 'white'))
    for (const c of cats) out.push(line(c.name, 'gray', c.measurable || c.kind === 'dns' ? categorySize(c) : tx('unknown')))
    const total = cats.reduce((s, c) => s + c.bytes, 0)
    out.push(line(tx('Space freed: about {0}', { size: total }), 'green'))
    out.push(line(tx('Free on {1} now: {0}', { size: freeBefore }, sys)))
    out.push(line(tx('Free on {1} after: about {0}', { size: freeBefore + total }, sys)))
    const locked = lockedTotal(categories, selected)
    if (locked) out.push(line(tx('Locked: {0} (files in use - not deleted, not included)', { size: locked }), 'yellow'))
    out.push(line(tx('(some files may be in use - they are skipped and the gain will be smaller)'), 'darkGray'))
  }
  if (lefts.length) {
    out.push(line(tx('Leftovers of uninstalled programs and games:'), 'white'))
    for (const l of lefts) out.push(line(leftoverLabel(l), 'gray', leftoverSize(l)))
    const bin = lefts.filter(goesToRecycleBin)
    if (bin.length) {
      const bytes = bin.reduce((s, l) => s + l.bytes, 0)
      out.push(line(tx('To the Recycle Bin: {0} ({1}) - the space is freed when you empty the Recycle Bin', String(bin.length), { size: bytes })))
    }
    const reg = lefts.filter((l) => l.kind === 'registry' || l.kind === 'regvalue').length
    if (reg) out.push(line(tx('Registry keys: {0} - .reg backup in logs\\kopie_rejestru + System Restore point', String(reg))))
    const tasks = lefts.filter((l) => l.kind === 'task').length
    if (tasks) out.push(line(tx('Scheduled tasks: {0} - .xml backup in logs\\kopie_rejestru', String(tasks))))
  }
  for (const n of notes(cats, lefts)) out.push(line(n, 'yellow'))
  return out
}

function notes(cats: Category[], lefts: Leftover[]): Text[] {
  const n: Text[] = []
  const has = (k: string) => cats.some((c) => c.kind === k)
  if (cats.some((c) => c.id === 'recyclebin')) n.push(tx('The Recycle Bin is emptied PERMANENTLY (before leftovers are moved to it).'))
  if (has('thumbnails')) n.push(tx('File Explorer restarts - the taskbar disappears for a few seconds.'))
  if (has('dism')) n.push(tx('DISM takes a few minutes; older updates can no longer be uninstalled afterwards.'))
  if (has('driverstore')) n.push(tx('Removed older driver versions cannot be brought back - a System Restore point is created first.'))
  if (has('winold')) n.push(tx('After removing Windows.old you cannot go back to the previous Windows version.'))
  if (cats.some((c) => c.hasProcesses)) n.push(tx('Do not open a browser while cleaning.'))
  if (lefts.some((l) => l.confidence === 'low')) n.push(tx('Low-confidence items are selected (game saves / large files) - make sure you do not need them.'))
  return n
}
