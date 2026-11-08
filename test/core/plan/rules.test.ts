import { describe, expect, it } from 'vitest'

import { isLarge, lockedTotal, recommended, restorePointNeeded, selectedTotal } from '../../../src/core/plan/rules'
import { category, GB, leftover } from '../../fixtures'

describe('recommended plan', () => {
  const cats = [
    category({ id: 'temp', bytes: 10 }),
    category({ id: 'empty', bytes: 0 }),
    category({ id: 'winold', bytes: 5 * GB, optIn: true, kind: 'winold' }),
    category({ id: 'www', bytes: 7, blocker: { key: 'a browser is running' } }),
    category({ id: 'recyclebin', bytes: 9, kind: 'recyclebin' }),
    category({ id: 'thumbnails', bytes: 3, kind: 'thumbnails' }),
  ]
  const lefts = [
    leftover({ id: 'R1', confidence: 'high', bytes: 1 }),
    leftover({ id: 'R2', confidence: 'medium', kind: 'registry' }),
    leftover({ id: 'R3', confidence: 'low' }),
    leftover({ id: 'R4', confidence: 'high', bytes: 2 * GB }),
    leftover({ id: 'R5', confidence: 'low', kind: 'file', largeFile: true, bytes: 100 }),
  ]

  it('selects non-empty folder categories, never opt-in, blocked or the Recycle Bin', () => {
    const r = recommended(cats, [])
    expect([...r].sort()).toEqual(['temp', 'thumbnails'])
  })

  it('selects high and medium leftovers but never large ones', () => {
    expect([...recommended([], lefts)].sort()).toEqual(['R1', 'R2'])
  })

  it('large items', () => {
    expect(isLarge(lefts[3])).toBe(true)
    expect(isLarge(lefts[4])).toBe(true)
    expect(isLarge(lefts[0])).toBe(false)
  })

  it('totals and the restore point rule', () => {
    const sel = new Set(['temp', 'R1'])
    expect(selectedTotal(cats, lefts, sel)).toBe(11)
    expect(lockedTotal([category({ id: 'a', locked: 4 })], new Set(['a']))).toBe(4)
    expect(restorePointNeeded(cats, lefts, sel)).toBe(false)
    expect(restorePointNeeded(cats, lefts, new Set(['R2']))).toBe(true)
    expect(restorePointNeeded([category({ id: 'driverstore', kind: 'driverstore' })], [], new Set(['driverstore']))).toBe(true)
  })
})
