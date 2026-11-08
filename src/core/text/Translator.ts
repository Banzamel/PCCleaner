import type { Arg, Text } from '../model/types'
import { fill, formatSize, renderText } from './format'

export type Dictionary = Record<string, string>

/**
 * Texts are written in English in the code; other languages map the English text to the translation
 * (src/lang/<code>.json). A text missing there is shown in English.
 */
export class Translator {
  constructor(
    readonly language: string,
    private readonly dict: Dictionary,
  ) {}

  lookup = (key: string): string => this.dict[key] ?? key

  /** Translates and fills {0}, {1}... */
  t = (key: string, ...args: Array<string | number>): string =>
    fill(this.lookup(key), args.map(String), this.language)

  /** Renders a Text coming from the core. */
  text = (text: Text | null | undefined): string => (text ? renderText(text, this.lookup, this.language) : '')

  size = (bytes: number): string => formatSize(bytes, this.language)

  arg = (a: Arg): string => (typeof a === 'string' ? a : 'size' in a ? this.size(a.size) : this.text(a.text))
}
