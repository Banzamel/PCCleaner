import type { Arg, Text } from '../model/types'

const GB = 1024 ** 3
const MB = 1024 ** 2

const LOCALES: Record<string, string> = { pl: 'pl-PL', en: 'en-US', es: 'es-ES' }

export function locale(language: string): string {
  return LOCALES[language] ?? 'en-US'
}

/** Size like the old program: "1.23 GB", "45 MB", "12 KB" or "0", in the user's number format. */
export function formatSize(bytes: number, language: string): string {
  const n = (v: number, digits: number) =>
    new Intl.NumberFormat(locale(language), { minimumFractionDigits: digits, maximumFractionDigits: digits }).format(v)
  if (bytes >= GB) return `${n(bytes / GB, 2)} GB`
  if (bytes >= MB) return `${n(bytes / MB, 0)} MB`
  if (bytes > 0) return `${n(bytes / 1024, 0)} KB`
  return '0'
}

/** Whole number with grouping ({0:N0} in the texts). */
export function formatCount(value: number, language: string): string {
  return new Intl.NumberFormat(locale(language), { maximumFractionDigits: 0 }).format(value)
}

/** Fills {0}, {1:N0}... with already rendered values. */
export function fill(template: string, values: string[], language: string): string {
  return template.replace(/\{(\d+)(:N0)?\}/g, (whole, i: string, n0?: string) => {
    const v = values[Number(i)]
    if (v === undefined) return whole
    if (n0 && /^\d+$/.test(v)) return formatCount(Number(v), language)
    return v
  })
}

export type Lookup = (key: string) => string

/** Renders a Text from the core: translated key, arguments formatted and translated recursively. */
export function renderText(text: Text, lookup: Lookup, language: string): string {
  const args = (text.args ?? []).map((a: Arg) => renderArg(a, lookup, language))
  return fill(lookup(text.key), args, language)
}

function renderArg(a: Arg, lookup: Lookup, language: string): string {
  if (typeof a === 'string') return a
  if ('size' in a) return formatSize(a.size, language)
  return renderText(a.text, lookup, language)
}

/** A Text built in TypeScript. */
export function tx(key: string, ...args: Arg[]): Text {
  return { key, args }
}
