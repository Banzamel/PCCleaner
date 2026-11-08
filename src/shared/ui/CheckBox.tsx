interface Props {
  checked: boolean
  disabled?: boolean
  label: string
  onToggle?: () => void
}

/** The square tick box of the lists (drawn, so it matches both themes). */
export function CheckBox({ checked, disabled, label, onToggle }: Props) {
  return (
    <button
      type="button"
      role="checkbox"
      aria-checked={checked}
      aria-label={label}
      disabled={disabled}
      className={`tick ${checked ? 'on' : ''}`}
      onClick={(e) => {
        e.stopPropagation()
        onToggle?.()
      }}
    >
      {checked ? '✓' : ''}
    </button>
  )
}
