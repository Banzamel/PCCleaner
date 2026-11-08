import type { Category, Leftover, StepState, Text } from '../model/types'
import { tx } from '../text/format'
import { goesToRecycleBin, restorePointNeeded } from './rules'

export interface Step {
  id: string
  name: Text
  /** Shown while waiting or running (e.g. "takes a few minutes"). */
  description: Text | null
  state: StepState
  bytes: number
  note: Text | null
}

/** The steps of the progress screen, in the order the cleaner runs them (same ids as Rust). */
export function initialSteps(categories: Category[], leftovers: Leftover[], selected: Set<string>): Step[] {
  const steps: Step[] = []
  const step = (id: string, name: Text, description: Text | null = null) =>
    steps.push({ id, name, description, state: 'waiting', bytes: 0, note: null })
  if (restorePointNeeded(categories, leftovers, selected)) step('restorepoint', tx('System Restore point'))
  for (const c of categories.filter((c) => selected.has(c.id))) {
    const d =
      c.kind === 'thumbnails' ? tx('the taskbar disappears for a moment') : c.kind === 'dism' || c.kind === 'winold' ? tx('takes a few minutes') : null
    step(c.id, c.name, d)
  }
  const sel = leftovers.filter((l) => selected.has(l.id))
  const bin = sel.filter(goesToRecycleBin).length
  if (bin) step('leftovers-bin', tx('Leftovers to the Recycle Bin: {0}', String(bin)))
  if (sel.length - bin) step('leftovers-backup', tx('Registry and tasks with a backup: {0}', String(sel.length - bin)))
  return steps
}

export function applyStep(steps: Step[], id: string, state: StepState, bytes: number, note: Text | null): Step[] {
  return steps.map((s) => (s.id !== id ? s : state === 'running' ? { ...s, state } : { ...s, state, bytes, note }))
}

/** Freed so far: everything except what went to the Recycle Bin (it is freed when emptied). */
export function freedSoFar(steps: Step[]): number {
  return steps.filter((s) => s.id !== 'leftovers-bin').reduce((sum, s) => sum + s.bytes, 0)
}

export function doneCount(steps: Step[]): number {
  return steps.filter((s) => s.state !== 'waiting' && s.state !== 'running').length
}
