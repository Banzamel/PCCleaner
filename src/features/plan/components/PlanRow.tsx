import { useServices, useUi } from '../../../app/AppContext'
import { categorySize, leftoverLabel, leftoverSize } from '../../../core/plan/describe'
import { itemBytes, itemId, type PlanItem } from '../../../core/plan/listing'
import { goesToRecycleBin } from '../../../core/plan/rules'
import { useT } from '../../../shared/i18n/useT'
import { Bar } from '../../../shared/ui/Bar'
import { CheckBox } from '../../../shared/ui/CheckBox'
import { Icon } from '../../../shared/ui/Icon'
import { Pill } from '../../../shared/ui/Pill'
import { PlanDetails } from './PlanDetails'
import { badgeOf, confidenceOf } from './badges'

interface Props {
  item: PlanItem
  open: boolean
  onOpen: () => void
}

/** Box (ticks), type icon, name with a second line, size bar, confidence. A click on the name opens the details. */
export function PlanRow({ item, open, onOpen }: Props) {
  const tr = useT()
  const { plan, ui } = useServices()
  const selected = useUi((s) => s.selected.has(itemId(item)))
  const cleaned = useUi((s) => s.cleaned)
  const ai = useUi((s) => (item.cat ? undefined : s.ai[item.l.id]))
  const max = useUi((s) => Math.max(1, ...[...(s.report?.categories ?? []), ...(s.report?.leftovers ?? [])].map((x) => x.bytes)))
  const blocked = item.cat && !!item.c.blocker && !selected
  const badge = badgeOf(item)
  const name = item.cat ? tr.text(item.c.name) : tr.text(leftoverLabel(item.l))
  let second = { text: '', tone: 'dim' }
  if (item.cat) {
    if (item.c.blocker) second = { text: `! ${tr.text(item.c.blocker)}`, tone: 'warning' }
    else second = { text: tr.text(item.c.note ?? item.c.description), tone: 'dim' }
  } else if (ai) second = { text: `${tr.t('AI (may be wrong)')}: ${ai}`, tone: 'ai' }
  else second = { text: tr.text(item.l.reason), tone: 'dim' }
  const measurable = item.cat ? item.c.measurable : goesToRecycleBin(item.l)
  const conf = confidenceOf(item)
  const reveal = () => !item.cat && goesToRecycleBin(item.l) && void ui.api.openTarget('reveal', item.l.path)
  return (
    <div className={`plan-row ${open ? 'open' : ''}`} onDoubleClick={reveal}>
      <CheckBox checked={selected} disabled={blocked || cleaned} label={name} onToggle={() => plan.toggle(itemId(item))} />
      <span className={`badge badge-${badge.tone}`} title={badge.label}>
        <Icon name={badge.icon} />
      </span>
      <button type="button" className="row-main" onClick={onOpen}>
        <strong className={blocked ? 'dim' : ''}>{name}</strong>
        <span className={`small tone-${second.tone}`}>{second.text}</span>
      </button>
      <span className="row-size">
        {measurable ? <Bar value={itemBytes(item) / max} tone={selected ? 'accent' : 'grey'} size="thin" /> : <span />}
        <strong className={measurable ? '' : 'dim'}>{tr.text(item.cat ? categorySize(item.c) : leftoverSize(item.l))}</strong>
      </span>
      <span className="row-conf">{conf ? <Pill tone={conf.tone}>{tr.t(conf.label)}</Pill> : null}</span>
      {open ? <PlanDetails item={item} ai={ai} /> : null}
    </div>
  )
}
