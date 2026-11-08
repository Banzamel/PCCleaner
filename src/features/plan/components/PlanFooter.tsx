import { useServices, useUi } from '../../../app/AppContext'
import { isLarge, lockedTotal, selectedTotal } from '../../../core/plan/rules'
import { useT } from '../../../shared/i18n/useT'
import { Button } from '../../../shared/ui/Button'
import { Card } from '../../../shared/ui/Card'

/** Selected total and the two actions: AI descriptions and "Clean selected...". */
export function PlanFooter() {
  const tr = useT()
  const { clean, ui, assistant } = useServices()
  const s = useUi((st) => st)
  const r = s.report
  const sel = r ? [...r.categories, ...r.leftovers].filter((x) => s.selected.has(x.id)) : []
  let note = tr.t('Items: {0}', String(sel.length))
  const big = r ? r.leftovers.filter((l) => isLarge(l) && s.selected.has(l.id)).length : 0
  if (big) note += ` · ${tr.t('of which large: {0}', String(big))}`
  const locked = r ? lockedTotal(r.categories, s.selected) : 0
  if (locked) note += ` · ${tr.t('{0} locked by running programs', tr.size(locked))}`
  const explain = () =>
    ui.busy(async () => {
      ui.status({ key: 'The model is describing the leftovers (about 20-40 s)...' })
      const n = await assistant.describeLeftovers()
      ui.status(n >= 0 ? { key: '{0} items described (second line, AI note).', args: [String(n)] } : { key: 'The assistant is unavailable - see the Log.' })
    })
  return (
    <Card className="plan-footer">
      <div className="footer-texts">
        <strong className="medium">{tr.t('Selected: {0}', tr.size(r ? selectedTotal(r.categories, r.leftovers, s.selected) : 0))}</strong>
        <span className="dim small">{note}</span>
      </div>
      <Button look="secondary" disabled={!r || s.busy || s.info.noAi || !r.leftovers.length} onClick={() => void explain()}>
        {tr.t('AI: what were the leftovers?')}
      </Button>
      <Button disabled={!r || s.busy || s.info.preview || s.cleaned} onClick={() => void clean.confirmAndClean()}>
        {s.info.preview ? tr.t('Preview - cleaning disabled') : tr.t('Clean selected...')}
      </Button>
    </Card>
  )
}
