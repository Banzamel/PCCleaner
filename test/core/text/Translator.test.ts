import { describe, expect, it } from 'vitest'

import { fill, formatSize } from '../../../src/core/text/format'
import { Translator } from '../../../src/core/text/Translator'

describe('texts', () => {
  it('formats sizes like the old program, in the user number format', () => {
    expect(formatSize(1.5 * 1024 ** 3, 'en')).toBe('1.50 GB')
    expect(formatSize(1.5 * 1024 ** 3, 'pl')).toBe('1,50 GB')
    expect(formatSize(12 * 1024 ** 2, 'en')).toBe('12 MB')
    expect(formatSize(2048, 'en')).toBe('2 KB')
    expect(formatSize(0, 'en')).toBe('0')
  })

  it('fills placeholders including the N0 format', () => {
    expect(fill('scanned {0:N0} files ({1:N0} s)...', ['12345', '7'], 'en')).toBe('scanned 12,345 files (7 s)...')
    expect(fill('{1} of {0} {2}', ['a', 'b'], 'en')).toBe('b of a {2}')
  })

  it('translates core texts with nested and size arguments, falling back to English', () => {
    const tr = new Translator('pl', { 'Recycle Bin': 'Kosz', '{0} freed': 'Zwolniono {0}' })
    expect(tr.text({ key: '{0}: {1}', args: [{ text: { key: 'Recycle Bin' } }, { size: 1024 ** 3 }] })).toBe('Kosz: 1,00 GB')
    expect(tr.t('{0} freed', '5 MB')).toBe('Zwolniono 5 MB')
    expect(tr.t('Missing text')).toBe('Missing text')
  })
})
