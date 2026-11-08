import { useUi } from '../../../app/AppContext'
import { doneCount, freedSoFar } from '../../../core/plan/steps'
import { useT } from '../../../shared/i18n/useT'
import { Bar } from '../../../shared/ui/Bar'
import { Card } from '../../../shared/ui/Card'

/** "Step 3 of 7 · name", freed so far and the overall bar. */
export function ProgressCard() {
  const tr = useT()
  const steps = useUi((s) => s.steps)
  const result = useUi((s) => s.cleanResult)
  const n = steps.length
  const done = doneCount(steps)
  const current = steps.find((s) => s.state === 'running')
  const title = result
    ? tr.t('Done - all steps finished')
    : current
      ? tr.t('Step {0} of {1} · {2}', String(done + 1), String(n), tr.text(current.name))
      : tr.t('Step {0} of {1}', String(Math.min(n, done + 1)), String(n))
  return (
    <Card className="progress-card">
      <div className="card-title-row">
        <h2>{title}</h2>
        <span className="dim">{tr.t('{0} freed', tr.size(freedSoFar(steps)))}</span>
      </div>
      <Bar value={n ? done / n : 0} tone={result ? 'green' : 'accent'} size="thick" />
    </Card>
  )
}
