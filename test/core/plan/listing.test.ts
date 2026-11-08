import { describe, expect, it } from 'vitest'

import { groupTotal, groups, matches, sortItems, tabCount, type PlanView } from '../../../src/core/plan/listing'
import { category, GB, leftover } from '../../fixtures'

const view: PlanView = { tab: 'plan', filter: 'all', search: '', sort: { key: 'size', desc: true } }
const render = (t: { key: string }) => t.key

describe('plan listing', () => {
  const cats = [category({ id: 'temp', bytes: 5 }), category({ id: 'dns', kind: 'dns', measurable: false }), category({ id: 'zero' })]
  const lefts = [leftover({ id: 'R1', bytes: 3 }), leftover({ id: 'R2', kind: 'registry' }), leftover({ id: 'R3', bytes: 2 * GB })]

  it('groups by kind and keeps large items on their own tab', () => {
    const g = groups(cats, lefts, 'plan')
    expect(g.map((x) => x.items.length)).toEqual([2, 1, 1])
    expect(groups(cats, lefts, 'large')[0].items.length).toBe(1)
    expect(tabCount(cats, lefts, 'plan')).toBe(3)
    expect(tabCount(cats, lefts, 'large')).toBe(1)
  })

  it('filters by selection, opt-in and search', () => {
    const items = groups(cats, lefts, 'plan').flatMap((g) => g.items)
    const sel = new Set(['R1'])
    expect(items.filter((i) => matches(i, { ...view, filter: 'selected' }, sel, render, {})).length).toBe(1)
    expect(items.filter((i) => matches(i, { ...view, search: 'r2' }, sel, render, {})).length).toBe(1)
    expect(items.filter((i) => matches(i, { ...view, search: 'steam' }, sel, render, { R1: 'A Steam game' })).length).toBe(1)
  })

  it('sorts by size and by name, and totals groups', () => {
    const items = groups(cats, lefts, 'plan')[0].items
    expect(sortItems(items, { key: 'size', desc: true }, () => '').map((i) => (i.cat ? i.c.id : '')).at(0)).toBe('temp')
    expect(sortItems(items, { key: 'name', desc: false }, (i) => (i.cat ? i.c.id : '')).map((i) => (i.cat ? i.c.id : ''))).toEqual(['dns', 'temp'])
    expect(groupTotal(items, new Set()).key).toBe('nothing selected')
    expect(groupTotal(items, new Set(['temp'])).args).toEqual([{ size: 5 }])
    expect(groupTotal(items, new Set(['dns'])).key).toBe('selected: {0}')
  })
})
