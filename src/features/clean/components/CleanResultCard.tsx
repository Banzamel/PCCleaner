import { useServices, useUi } from '../../../app/AppContext'
import { freedSoFar } from '../../../core/plan/steps'
import { useT } from '../../../shared/i18n/useT'
import { Bar } from '../../../shared/ui/Bar'
import { Button } from '../../../shared/ui/Button'
import { Card } from '../../../shared/ui/Card'

/** Space freed, free space before and after as two bars, backups, the restore point. */
export function CleanResultCard() {
  const tr = useT()
  const { scan, ui } = useServices()
  const r = useUi((s) => s.cleanResult)
  const steps = useUi((s) => s.steps)
  const sys = useUi((s) => s.info.systemDrive)
  const info: string[] = []
  if (r) {
    info.push(r.backupCount ? tr.t('Registry and task backups: {0} files ({1})', String(r.backupCount), r.backups ?? '') : tr.t('Registry and task backups: not needed'))
    info.push(r.restorePoint ? tr.t('Restore point: {0}', tr.text(r.restorePoint)) : tr.t('Restore point: not needed'))
    if (r.locked) info.push(tr.t('Skipped: {0} in use', tr.size(r.locked)))
    if (r.inRecycleBin) info.push(tr.t('In the Recycle Bin: {0} - freed when you empty it', tr.size(r.inRecycleBin)))
  }
  const max = r ? Math.max(1, r.freeBefore, r.freeAfter) : 1
  const sec = r ? Math.round(r.durationMs / 1000) : 0
  return (
    <Card className="result">
      <h2>{r ? tr.t('Done') : tr.t('When it finishes you will see')}</h2>
      <div className={`result-box ${r ? 'success' : ''}`}>
        <p className="small">{r ? tr.t('Freed') : tr.t('Freed so far')}</p>
        <p className="result-big">{tr.size(r ? r.total : freedSoFar(steps))}</p>
        <p className="small">
          {r
            ? tr.t('Free on {0} {1} → {2} · {3} min {4} s', sys, tr.size(r.freeBefore), tr.size(r.freeAfter), String(Math.floor(sec / 60)), String(sec % 60))
            : tr.t('Free space before and after appears after the last step.')}
        </p>
        {r ? (
          <div className="free-bars">
            <span>{tr.t('before')}</span>
            <Bar value={r.freeBefore / max} tone="grey" />
            <strong>{tr.size(r.freeBefore)}</strong>
            <span>{tr.t('after')}</span>
            <Bar value={r.freeAfter / max} tone="green" />
            <strong>{tr.size(r.freeAfter)}</strong>
          </div>
        ) : null}
      </div>
      <div className="dim small result-info">
        {(r ? info : [tr.t('Registry and task backups, the restore point and skipped files appear here after the last step.')]).map((t) => (
          <p key={t}>{t}</p>
        ))}
      </div>
      {r ? (
        <div className="row-buttons">
          <Button onClick={() => void scan.scan()}>{tr.t('Scan again')}</Button>
          <Button look="secondary" onClick={() => void ui.api.openTarget('log')}>
            {tr.t('Open the log')}
          </Button>
        </div>
      ) : null}
    </Card>
  )
}
