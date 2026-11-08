import { leftoverLabel, leftoverSize } from '../../../core/plan/describe'
import type { PlanItem } from '../../../core/plan/listing'
import { goesToRecycleBin } from '../../../core/plan/rules'
import { useT } from '../../../shared/i18n/useT'

/** The full description of an item, shown after a click on its name. */
export function PlanDetails({ item, ai }: { item: PlanItem; ai?: string }) {
  const tr = useT()
  const lines: string[] = []
  if (item.cat) {
    const c = item.c
    lines.push(tr.text(c.name), tr.text(c.description))
    if (c.note) lines.push(tr.text(c.note))
    if (c.blocker) lines.push(tr.text(c.blocker))
  } else {
    const l = item.l
    lines.push(tr.text(leftoverLabel(l)), tr.text(l.reason))
    if (ai) lines.push(`${tr.t('AI (may be wrong)')}: ${ai}`)
    const info = [`${tr.t('Size')}: ${tr.text(leftoverSize(l))}`, `${tr.t('Confidence')}: ${tr.t(l.confidence)}`]
    if (l.lastWrite) info.push(`${tr.t('Last changed')}: ${l.lastWrite}`)
    lines.push(info.join(' · '))
    if (goesToRecycleBin(l)) lines.push(tr.t('Double-click shows the item in File Explorer.'))
  }
  return (
    <div className="row-details">
      {lines.filter(Boolean).map((t, i) => (
        <p key={i}>{t}</p>
      ))}
    </div>
  )
}
