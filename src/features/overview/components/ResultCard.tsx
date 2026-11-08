import { useServices, useUi } from '../../../app/AppContext'
import { selectedTotal } from '../../../core/plan/rules'
import { useT } from '../../../shared/i18n/useT'
import { Button } from '../../../shared/ui/Button'
import { Card } from '../../../shared/ui/Card'
import { StatTiles } from './StatTiles'

/** Small caption, big number, description, buttons, and the three tiles on the right. */
export function ResultCard() {
  const tr = useT()
  const { scan, ui, clean } = useServices()
  const s = useUi((st) => st)
  const r = s.report
  const total = r ? selectedTotal(r.categories, r.leftovers, s.selected) : 0
  let small = tr.t('First step')
  let big = tr.t('Ready to scan')
  let tone = ''
  let stage = tr.t('A scan usually takes 1-3 minutes and deletes nothing. Then you get a cleanup plan to review.')
  if (s.scanning) {
    small = tr.t('Scanning - nothing is deleted')
    big = tr.t('Scanning...')
    stage = s.stage ? tr.text(s.stage) : ''
  } else if (!r && s.stage) {
    big = tr.t('The scan failed')
    tone = 'red'
    stage = tr.text(s.stage)
  } else if (r && s.cleaned) {
    small = tr.t('Cleaning finished')
    big = tr.t('{0} freed', tr.size(s.cleanResult?.total ?? 0))
    tone = 'green'
    stage = tr.t('Click "Scan again" to see the current state of your drives.')
  } else if (r && total > 0) {
    small = tr.t('Can be freed safely')
    big = tr.size(total)
    tone = 'accent'
    stage = tr.t('Only things that rebuild themselves or go to the Recycle Bin are selected. Nothing is deleted without your confirmation.')
  } else if (r) {
    small = tr.t('Scan result')
    big = tr.t('Your PC is clean')
    tone = 'green'
    stage = tr.t('Nothing is selected. The cleanup plan also lists opt-in items.')
  }
  const showStart = !r || s.cleaned
  return (
    <Card className="result-card">
      <div className="result-left">
        <p className="dim">{small}</p>
        <p className={`result-big tone-${tone}`}>{big}</p>
        <p className="dim result-stage">{stage}</p>
        <div className="row-buttons">
          {showStart ? (
            <Button disabled={s.busy} onClick={() => void scan.scan()}>
              {r ? tr.t('Scan again') : tr.t('Scan now')}
            </Button>
          ) : (
            <>
              <Button onClick={() => ui.go('plan')}>{tr.t('View the cleanup plan')}</Button>
              <Button look="secondary" disabled={s.busy || s.info.preview} onClick={() => void clean.confirmAndClean()}>
                {s.info.preview ? tr.t('Preview - no cleaning') : tr.t('Clean now...')}
              </Button>
            </>
          )}
        </div>
      </div>
      {r && !s.scanning ? <StatTiles /> : null}
    </Card>
  )
}
