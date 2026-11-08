interface Props {
  /** 0..1 */
  value: number
  tone?: 'accent' | 'grey' | 'green' | 'warning' | 'red'
  size?: 'thin' | 'normal' | 'thick'
}

/** A rounded size bar on a track. */
export function Bar({ value, tone = 'accent', size = 'normal' }: Props) {
  const pct = Math.max(0, Math.min(1, value)) * 100
  return (
    <span className={`bar bar-${size}`}>
      <span className={`bar-fill bar-${tone}`} style={{ width: `${Math.max(pct, 3)}%` }} />
    </span>
  )
}
