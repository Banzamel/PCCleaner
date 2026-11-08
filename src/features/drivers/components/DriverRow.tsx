import { useState } from 'react'

import { useServices, useUi } from '../../../app/AppContext'
import type { DriverRow as Row } from '../../../app/state'
import { useT } from '../../../shared/i18n/useT'
import { CheckBox } from '../../../shared/ui/CheckBox'
import { Pill, type PillTone } from '../../../shared/ui/Pill'
import { selectable } from '../DriversService'

const STATE: Record<string, { label: string; tone: PillTone }> = {
  new: { label: 'available', tone: 'blue' },
  waiting: { label: 'waiting', tone: 'neutral' },
  searching: { label: 'preparing', tone: 'accent' },
  downloading: { label: 'downloading', tone: 'accent' },
  installing: { label: 'installing', tone: 'accent' },
  ok: { label: 'installed', tone: 'ok' },
  restart: { label: 'after restart', tone: 'warn' },
  gone: { label: 'no longer offered', tone: 'error' },
  fail: { label: 'failed', tone: 'error' },
}

const version = (v: string | null, d: string | null) => [v, d ? `(${d})` : ''].filter(Boolean).join(' ')

export function DriverRow({ driver: d }: { driver: Row }) {
  const tr = useT()
  const { drivers } = useServices()
  const preview = useUi((s) => s.info.preview)
  const busy = useUi((s) => s.busy)
  const [open, setOpen] = useState(false)
  const state = STATE[d.state] ?? STATE.fail
  const now = d.currentVersion ? version(d.currentVersion, d.currentDate) : d.problem ? tr.text(d.problem) : tr.t('unknown')
  return (
    <div className="driver-row">
      <CheckBox checked={d.selected} disabled={busy || !selectable(d, preview)} label={d.name} onToggle={() => drivers.toggle(d.id)} />
      <button type="button" className="row-main" onClick={() => setOpen(!open)}>
        <strong className={d.state === 'ok' || d.state === 'restart' ? 'dim' : ''}>{d.name}</strong>
        <span className="small dim">{[d.provider, d.class].filter(Boolean).join(' · ')}</span>
        {d.state === 'fail' ? (
          <span className="small tone-red">{tr.t('Failed: {0}', d.error)}</span>
        ) : (
          <span className="small">{tr.t('Now: {0}   →   new: {1}', now, version(d.version, d.newDate))}</span>
        )}
        {open ? <span className="small dim">{[d.title, d.model].filter(Boolean).join(' · ')}</span> : null}
      </button>
      <span className="dim">{d.bytes > 0 ? tr.size(d.bytes) : ''}</span>
      <Pill tone={state.tone}>{tr.t(state.label)}</Pill>
    </div>
  )
}
