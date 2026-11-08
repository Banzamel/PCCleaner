import { describe, expect, it } from 'vitest'

import { ANALYSIS_ADVICE, analysisPrompt, fixedAdvice } from '../../../src/core/assistant/advicePrompts'
import { CHAT_TURNS, Conversation } from '../../../src/core/assistant/Conversation'
import { chatReport } from '../../../src/core/assistant/report'
import { parseItems, withAdvice } from '../../../src/core/assistant/schemas'
import { Translator } from '../../../src/core/text/Translator'
import es from '../../../src/lang/es.json'
import { emptyProfile, leftover } from '../../fixtures'

describe('assistant', () => {
  it('sets the advice itself for system, program and TEMP folders', () => {
    expect(ANALYSIS_ADVICE[fixedAdvice('C:\\Windows\\WinSxS') ?? -1]).toBe('leave it - part of Windows')
    expect(ANALYSIS_ADVICE[fixedAdvice('C:\\Program Files\\Vendor') ?? -1]).toBe('uninstall in Settings > Apps')
    expect(ANALYSIS_ADVICE[fixedAdvice('C:\\Users\\Jan\\AppData\\Local\\Temp') ?? -1]).toBe('the TEMP plan category cleans it')
    expect(fixedAdvice('D:\\Projects')).toBeNull()
  })

  it('gives Spanish the English prompt answered in Spanish with translated advice', () => {
    const p = analysisPrompt(new Translator('es', es))
    expect(p).toContain('in Spanish')
    expect(p).not.toContain('"leave it - part of Windows"')
  })

  it('parses structured answers and drops broken items', () => {
    const items = parseItems('{"items":[{"id":"A1","description":"Steam game","advice":"x"},{"id":"A2"},7]}', 'items')
    expect(items).toEqual([{ id: 'A1', description: 'Steam game', advice: 'x' }])
    expect(withAdvice('A game.;', 'Advice', 'keep it on')).toBe('A game. Advice: keep it on.')
  })

  it('trims the chat report to the limit, largest items first', () => {
    const lefts = Array.from({ length: 200 }, (_, i) => leftover({ id: `R${i}`, bytes: i, path: `C:\\${'x'.repeat(100)}\\${i}` }))
    const tr = new Translator('en', {})
    const json = chatReport({ categories: [], leftovers: lefts, selected: new Set(), profile: emptyProfile, analyses: [], startup: [], ai: {} }, tr, 8000)
    const r = JSON.parse(json)
    expect(json.length).toBeLessThanOrEqual(8000)
    expect(r.leftovers[0].id).toBe('R199')
    expect(r.summary.leftovers_hidden).toBe(200 - r.leftovers.length)
  })

  it('keeps only the last exchanges', () => {
    const c = new Conversation('system')
    for (let i = 0; i < 20; i++) c.record(`q${i}`, `a${i}`)
    const m = c.request('now')
    expect(m[0].content).toBe('system')
    expect(m.length).toBe(1 + 2 * CHAT_TURNS + 1)
    expect(m.at(-1)?.content).toBe('now')
  })
})
