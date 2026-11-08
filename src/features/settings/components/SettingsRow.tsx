import type { ReactNode } from 'react'

/** Description on the left, control on the right. */
export function SettingsRow({ title, description, children }: { title: string; description: string; children: ReactNode }) {
  return (
    <div className="settings-row">
      <div className="settings-text">
        <strong>{title}</strong>
        <span className="dim small">{description}</span>
      </div>
      <div className="settings-control">{children}</div>
    </div>
  )
}
