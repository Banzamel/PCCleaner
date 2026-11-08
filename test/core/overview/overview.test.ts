import { describe, expect, it } from 'vitest'

import { bars, tiles } from '../../../src/core/overview/overview'
import { category, GB, leftover } from '../../fixtures'

describe('overview', () => {
  it('tiles split safe, to review and large', () => {
    const cats = [category({ id: 'a', bytes: 10 }), category({ id: 'b', bytes: 4 })]
    const lefts = [leftover({ id: 'R1', bytes: 2 * GB })]
    const t = tiles(cats, lefts, new Set(['a']))
    expect(t.map((x) => [x.bytes, x.count])).toEqual([
      [10, 1],
      [4, 1],
      [2 * GB, 1],
    ])
    expect(t[2].tab).toBe('large')
  })

  it('bars keep six largest plus the rest', () => {
    const cats = Array.from({ length: 9 }, (_, i) => category({ id: `c${i}`, bytes: i + 1 }))
    const b = bars(cats, [leftover({ id: 'R1', bytes: 100 })], new Set(['c0']))
    expect(b.length).toBe(7)
    expect(b[0].name.key).toBe('Program leftovers')
    expect(b[6]).toMatchObject({ name: { key: 'Other items' }, bytes: 1 + 2 + 3 + 4, selected: true })
  })
})
