import { describe, expect, it } from 'vitest'

import { summary } from '../../../src/core/plan/summary'
import { category, leftover } from '../../fixtures'

describe('summary before cleaning', () => {
  it('is null when nothing is selected', () => {
    expect(summary([category({ id: 'temp' })], [], new Set(), 0, 'C:')).toBeNull()
  })

  it('lists categories, leftovers, backups and warnings', () => {
    const cats = [category({ id: 'recyclebin', kind: 'recyclebin', bytes: 5 }), category({ id: 'www', hasProcesses: true, bytes: 1 })]
    const lefts = [leftover({ id: 'R1', bytes: 3 }), leftover({ id: 'R2', kind: 'registry' }), leftover({ id: 'R3', kind: 'task', confidence: 'low' })]
    const lines = summary(cats, lefts, new Set(['recyclebin', 'www', 'R1', 'R2', 'R3']), 100, 'C:') ?? []
    const keys = lines.map((l) => l.text.key)
    expect(keys).toContain('Space freed: about {0}')
    expect(keys).toContain('To the Recycle Bin: {0} ({1}) - the space is freed when you empty the Recycle Bin')
    expect(keys.some((k) => k.startsWith('Registry keys'))).toBe(true)
    expect(keys.some((k) => k.startsWith('Scheduled tasks'))).toBe(true)
    expect(keys).toContain('The Recycle Bin is emptied PERMANENTLY (before leftovers are moved to it).')
    expect(keys).toContain('Do not open a browser while cleaning.')
    expect(keys.at(-1)).toBe('Low-confidence items are selected (game saves / large files) - make sure you do not need them.')
    expect(lines.find((l) => l.text.key === 'Space freed: about {0}')?.text.args).toEqual([{ size: 6 }])
  })
})
