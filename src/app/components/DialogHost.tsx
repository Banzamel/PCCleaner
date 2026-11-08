import { useEffect, useRef } from 'react'

import { useT } from '../../shared/i18n/useT'
import { Button } from '../../shared/ui/Button'
import { useServices, useUi } from '../AppContext'

/** Message boxes and confirmations. The default button of a confirmation is "No". */
export function DialogHost() {
  const dialog = useUi((s) => s.dialog)
  const { ui } = useServices()
  const tr = useT()
  const noRef = useRef<HTMLButtonElement>(null)
  useEffect(() => noRef.current?.focus(), [dialog])
  if (!dialog) return null
  const confirm = dialog.kind === 'confirm'
  return (
    <div className="dialog-backdrop" onKeyDown={(e) => e.key === 'Escape' && ui.close(false)}>
      <div className={`dialog dialog-${dialog.tone}`} role="dialog" aria-modal="true" aria-label={dialog.title}>
        <h2>{dialog.title}</h2>
        <div className="dialog-body">
          {dialog.lines.map((l, i) => (
            <div key={i} className={`dialog-line tone-${l.tone ?? 'gray'} ${l.value !== undefined ? 'with-value' : ''}`}>
              <span>{l.text}</span>
              {l.value !== undefined ? <span className="dialog-value">{l.value}</span> : null}
            </div>
          ))}
          {dialog.question ? <p className="dialog-question">{dialog.question}</p> : null}
        </div>
        <div className="dialog-buttons">
          {confirm ? (
            <>
              <Button look="primary" onClick={() => ui.close(true)}>
                {tr.t('Yes')}
              </Button>
              <Button look="outline" ref={noRef} onClick={() => ui.close(false)}>
                {tr.t('No')}
              </Button>
            </>
          ) : (
            <Button look="primary" ref={noRef} onClick={() => ui.close(true)}>
              OK
            </Button>
          )}
        </div>
      </div>
    </div>
  )
}
