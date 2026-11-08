import { useServices, useUi } from '../../../app/AppContext'
import type { StartupEntry } from '../../../core/model/types'
import { useT } from '../../../shared/i18n/useT'
import { Icon } from '../../../shared/ui/Icon'
import { Pill } from '../../../shared/ui/Pill'
import { Toggle } from '../../../shared/ui/Toggle'

const COLORS = ['#5865F2', '#0A64AD', '#1A7F3C', '#B30B00', '#7C3AED', '#C2410C', '#0F766E', '#BE185D']

/** A stable colour per name for the initial tile. */
export function initialColor(name: string): string {
  let h = 0
  for (const c of name) h = (h * 31 + c.charCodeAt(0)) % 1000003
  return COLORS[h % COLORS.length]
}

export function StartupRow({ entry: e }: { entry: StartupEntry }) {
  const tr = useT()
  const { startup, ui } = useServices()
  const ai = useUi((s) => s.ai[e.id])
  const preview = useUi((s) => s.info.preview)
  const busy = useUi((s) => s.busy)
  const third = e.dead
    ? { text: tr.t('Dead entry: the program is no longer on the drive. The bin next to the switch removes the entry.'), tone: 'warning' }
    : ai
      ? { text: `${tr.t('AI (may be wrong)')}: ${ai}`, tone: 'ai' }
      : e.protected
        ? { text: tr.text(e.protected), tone: 'dim' }
        : null
  const reveal = () => e.exists && e.file && void ui.api.openTarget('reveal', e.file)
  return (
    <div className="startup-row" onDoubleClick={reveal} title={tr.t('Double-click shows the file in File Explorer.')}>
      <span className="initial" style={{ background: e.protected ? 'var(--tile-protected)' : initialColor(e.name) }}>
        {(e.name[0] ?? '?').toUpperCase()}
      </span>
      <span className="row-main">
        <span className="startup-name">
          <strong className={e.isEnabled ? '' : 'dim'}>{e.name}</strong>
          {e.protected ? <Pill tone="neutral">{tr.text(e.protected).split(' - ')[0]}</Pill> : null}
        </span>
        <span className="small dim">{[e.publisher, e.command].filter(Boolean).join(' · ')}</span>
        {third ? <span className={`small tone-${third.tone}`}>{third.text}</span> : null}
      </span>
      <span className="small dim startup-source">{tr.text(e.source)}</span>
      {!e.protected && !preview ? (
        <button type="button" className={`bin ${e.dead ? 'tone-warning' : ''}`} aria-label={tr.t('Remove from startup')} disabled={busy} onClick={() => void startup.remove(e)}>
          <Icon name="bin" />
        </button>
      ) : (
        <span className="bin-space" />
      )}
      <Toggle on={e.isEnabled} disabled={!!e.protected || preview || busy} label={e.name} onToggle={() => void startup.toggle(e)} />
    </div>
  )
}
