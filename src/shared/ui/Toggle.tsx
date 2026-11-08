interface Props {
  on: boolean
  disabled?: boolean
  label: string
  onToggle: () => void
}

/** On/off switch (startup apps). */
export function Toggle({ on, disabled, label, onToggle }: Props) {
  return (
    <button type="button" role="switch" aria-checked={on} aria-label={label} disabled={disabled} className={`toggle ${on ? 'on' : ''}`} onClick={onToggle}>
      <span className="toggle-knob" />
    </button>
  )
}
