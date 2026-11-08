interface Option<T extends string> {
  id: T
  label: string
}

interface Props<T extends string> {
  options: Array<Option<T>>
  value: T
  onChange: (id: T) => void
}

/** Segmented switch for tabs and filters. */
export function Segment<T extends string>({ options, value, onChange }: Props<T>) {
  return (
    <div className="segment" role="tablist">
      {options.map((o) => (
        <button key={o.id} type="button" role="tab" aria-selected={o.id === value} className={o.id === value ? 'on' : ''} onClick={() => onChange(o.id)}>
          {o.label}
        </button>
      ))}
    </div>
  )
}
