export type PillTone = 'ok' | 'warn' | 'error' | 'neutral' | 'blue' | 'accent' | 'purple'

/** Small rounded label (confidence, driver state, protected). */
export function Pill({ tone, children }: { tone: PillTone; children: string }) {
  return <span className={`pill pill-${tone}`}>{children}</span>
}
