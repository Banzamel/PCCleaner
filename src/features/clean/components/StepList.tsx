import { useUi } from '../../../app/AppContext'
import type { Step } from '../../../core/plan/steps'
import { useT } from '../../../shared/i18n/useT'

const MARK: Record<string, string> = { ok: '✓', partial: '✓', running: '…', skipped: '!', fail: '!', waiting: '' }

/** Every step with its state: waiting, in progress, done, done with skips, skipped, failed. */
export function StepList() {
  const tr = useT()
  const steps = useUi((s) => s.steps)
  const note = (s: Step): string => {
    if (s.state === 'waiting') return s.description ? tr.t('waiting - {0}', tr.text(s.description)) : tr.t('waiting')
    if (s.state === 'running') return s.description ? tr.t('in progress - {0}', tr.text(s.description)) : tr.t('in progress...')
    return tr.text(s.note)
  }
  return (
    <ul className="step-list">
      {steps.map((s) => (
        <li key={s.id} className={`step step-${s.state}`}>
          <span className="step-mark">{MARK[s.state]}</span>
          <span className="step-text">
            <strong>{tr.text(s.name)}</strong>
            <span className="small step-note">{note(s)}</span>
          </span>
          <strong>{s.bytes > 0 ? tr.size(s.bytes) : ''}</strong>
        </li>
      ))}
    </ul>
  )
}
