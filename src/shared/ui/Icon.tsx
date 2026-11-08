/** Glyphs of the Windows 10/11 icon font (Segoe MDL2 Assets), the same ones the old window used. */
export const GLYPHS = {
  overview: 0xe80f,
  plan: 0xe74d,
  analysis: 0xeda2,
  startup: 0xe7e8,
  drivers: 0xe772,
  chat: 0xe8bd,
  log: 0xe81c,
  settings: 0xe713,
  info: 0xe946,
  search: 0xe721,
  category: 0xe895,
  folder: 0xe8b7,
  link: 0xe71b,
  file: 0xe8a5,
  task: 0xe823,
  registry: 0xe713,
  warning: 0xe7ba,
  bin: 0xe74d,
  lock: 0xe72e,
} as const

export type GlyphName = keyof typeof GLYPHS

export function Icon({ name, className }: { name: GlyphName; className?: string }) {
  return (
    <span className={`icon ${className ?? ''}`} aria-hidden="true">
      {String.fromCharCode(GLYPHS[name])}
    </span>
  )
}
