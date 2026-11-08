import type { PlanItem } from '../../../core/plan/listing'
import type { GlyphName } from '../../../shared/ui/Icon'
import type { PillTone } from '../../../shared/ui/Pill'

/** Type icon: cache that rebuilds itself, folder, file, shortcut, task, registry. */
export function badgeOf(i: PlanItem): { icon: GlyphName; tone: string; label: string } {
  if (i.cat) return { icon: 'category', tone: 'accent', label: 'CAT' }
  switch (i.l.kind) {
    case 'folder':
      return { icon: 'folder', tone: 'warn', label: 'DIR' }
    case 'file':
      return i.l.path.toLowerCase().endsWith('.lnk') ? { icon: 'link', tone: 'blue', label: 'LNK' } : { icon: 'file', tone: 'blue', label: 'FILE' }
    case 'task':
      return { icon: 'task', tone: 'ok', label: 'TASK' }
    default:
      return { icon: 'registry', tone: 'purple', label: 'REG' }
  }
}

/** Confidence pill; opt-in categories get "opt-in". */
export function confidenceOf(i: PlanItem): { label: string; tone: PillTone } | null {
  const c = i.cat ? (i.c.optIn ? 'optin' : null) : i.l.confidence
  if (!c) return null
  return {
    high: { label: 'high', tone: 'ok' as PillTone },
    medium: { label: 'medium', tone: 'warn' as PillTone },
    low: { label: 'low', tone: 'error' as PillTone },
    optin: { label: 'opt-in', tone: 'neutral' as PillTone },
  }[c]
}
