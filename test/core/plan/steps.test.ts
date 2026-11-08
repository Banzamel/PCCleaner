import { describe, expect, it } from 'vitest'

import { applyStep, doneCount, freedSoFar, initialSteps } from '../../../src/core/plan/steps'
import { category, leftover } from '../../fixtures'

describe('cleanup steps', () => {
  it('follow the order the cleaner runs them', () => {
    const steps = initialSteps(
      [category({ id: 'temp' }), category({ id: 'dism', kind: 'dism' })],
      [leftover({ id: 'R1' }), leftover({ id: 'R2', kind: 'registry' })],
      new Set(['temp', 'dism', 'R1', 'R2']),
    )
    expect(steps.map((s) => s.id)).toEqual(['restorepoint', 'temp', 'dism', 'leftovers-bin', 'leftovers-backup'])
    expect(steps[2].description?.key).toBe('takes a few minutes')
  })

  it('apply updates and count what is freed (not the Recycle Bin)', () => {
    let steps = initialSteps([category({ id: 'temp' })], [leftover({ id: 'R1' })], new Set(['temp', 'R1']))
    steps = applyStep(steps, 'temp', 'running', 0, null)
    expect(doneCount(steps)).toBe(0)
    steps = applyStep(steps, 'temp', 'ok', 100, null)
    steps = applyStep(steps, 'leftovers-bin', 'ok', 50, null)
    expect(doneCount(steps)).toBe(2)
    expect(freedSoFar(steps)).toBe(100)
  })
})
