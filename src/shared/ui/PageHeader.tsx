import { useState, type ReactNode } from 'react'

import { Icon } from './Icon'

interface Props {
  title: string
  /** Full description behind the "i" next to the title. */
  info: string
  /** Status line under the title (e.g. the last scan). */
  status?: string
  children?: ReactNode
}

/** Page title with an "i" that shows the page description, and buttons on the right. */
export function PageHeader({ title, info, status, children }: Props) {
  const [open, setOpen] = useState(false)
  return (
    <header className="page-header">
      <div className="page-header-text">
        <div className="page-title-row">
          <h1>{title}</h1>
          <button type="button" className="info-button" aria-expanded={open} aria-label={info} onClick={() => setOpen(!open)}>
            <Icon name="info" />
          </button>
        </div>
        {open ? (
          <p className="page-info" onClick={() => setOpen(false)}>
            {info}
          </p>
        ) : (
          <p className="page-status">{status}</p>
        )}
      </div>
      {children ? <div className="page-actions">{children}</div> : null}
    </header>
  )
}
