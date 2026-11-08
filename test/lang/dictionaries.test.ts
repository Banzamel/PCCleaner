import { readdirSync, readFileSync, statSync } from 'node:fs'
import { join } from 'node:path'

import { describe, expect, it } from 'vitest'

import es from '../../src/lang/es.json'
import pl from '../../src/lang/pl.json'

const ROOT = join(__dirname, '..', '..')

function files(dir: string, ext: RegExp): string[] {
  return readdirSync(dir).flatMap((f) => {
    const p = join(dir, f)
    return statSync(p).isDirectory() ? files(p, ext) : ext.test(f) ? [p] : []
  })
}

const unescape = (s: string) => s.replace(/\\"/g, '"').replace(/\\'/g, "'").replace(/\\\\/g, '\\').replace(/\\n/g, '\n')

/** Every literal text key used in the code: t('...'), tx('...'), { key: '...' } and Text::new("...") in Rust. */
export function usedKeys(): string[] {
  const keys = new Set<string>()
  for (const f of files(join(ROOT, 'src'), /\.tsx?$/)) {
    const src = readFileSync(f, 'utf8')
    for (const m of src.matchAll(/\b(?:t|tx)\(\s*'((?:[^'\\]|\\.)*)'/g)) keys.add(unescape(m[1]))
    for (const m of src.matchAll(/\bkey: '((?:[^'\\]|\\.)*)'/g)) keys.add(unescape(m[1]))
    for (const m of src.matchAll(/\b(?:label|name): '((?:[^'\\]|\\.)*)'/g)) if (f.includes('NavBar') || f.includes('scanItems') || f.includes('DriverRow')) keys.add(unescape(m[1]))
  }
  for (const f of files(join(ROOT, 'src-tauri', 'core', 'src'), /\.rs$/)) {
    const src = readFileSync(f, 'utf8')
    for (const m of src.matchAll(/Text::new\("((?:[^"\\]|\\.)*)"\)/g)) keys.add(unescape(m[1]))
    for (const m of src.matchAll(/^\s*\(r"[^"]*", "((?:[^"\\]|\\.)*)"\),$/gm)) keys.add(unescape(m[1]))
  }
  // Keys made only of placeholders and punctuation need no translation; sort keys are not texts
  const notTexts = new Set(['size', 'name'])
  return [...keys].filter((k) => !notTexts.has(k) && /[A-Za-z]{2}/.test(k.replace(/\{\d+(:N0)?\}/g, '')))
}

const placeholders = (s: string) => [...s.matchAll(/\{\d+(:N0)?\}/g)].map((m) => m[0].replace(':N0', '')).sort()

describe('dictionaries', () => {
  const keys = usedKeys()

  it.each([
    ['pl', pl as Record<string, string>],
    ['es', es as Record<string, string>],
  ])('%s has every text used in the code', (_, dict) => {
    const missing = keys.filter((k) => !(k in dict))
    expect(missing).toEqual([])
  })

  it.each([
    ['pl', pl as Record<string, string>],
    ['es', es as Record<string, string>],
  ])('%s keeps the placeholders of each text', (_, dict) => {
    const broken = Object.entries(dict).filter(([k, v]) => placeholders(k).join() !== placeholders(v).join())
    expect(broken).toEqual([])
  })
})
