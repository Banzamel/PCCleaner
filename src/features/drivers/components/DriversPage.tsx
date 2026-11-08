import { useServices, useUi } from '../../../app/AppContext'
import { useT } from '../../../shared/i18n/useT'
import { Button } from '../../../shared/ui/Button'
import { Card } from '../../../shared/ui/Card'
import { EmptyText } from '../../../shared/ui/EmptyText'
import { Icon } from '../../../shared/ui/Icon'
import { NumberTiles } from '../../../shared/ui/NumberTiles'
import { PageHeader } from '../../../shared/ui/PageHeader'
import { selectable } from '../DriversService'
import { DriverRow } from './DriverRow'

export function DriversPage() {
  const tr = useT()
  const { drivers, ui } = useServices()
  const s = useUi((st) => st)
  const list = s.drivers
  const chosen = (list ?? []).filter((d) => d.selected)
  const n = (v: number) => (list === null ? '-' : String(v))
  const empty = s.driversNote
    ? tr.text(s.driversNote)
    : list === null
      ? tr.t('Click "Check" to look for newer drivers on Windows Update. It usually takes 1-3 minutes and installs nothing.')
      : tr.t('Your drivers are up to date according to Windows Update and no device reports a problem.')
  const sorted = [...(list ?? [])].sort((a, b) => a.class.localeCompare(b.class) || a.name.localeCompare(b.name))
  return (
    <div className="drivers-page">
      <PageHeader
        title={tr.t('Drivers')}
        info={tr.t('Newer drivers for the devices in this PC from Windows Update - signed, the same as the optional updates. The box on the left ticks a driver; click its name for the full description. Double-click opens Device Manager.')}
      >
        <Button look="outline" onClick={() => void ui.api.openTarget('windows-update')}>
          Windows Update
        </Button>
        <Button disabled={s.busy} onClick={() => void drivers.check()}>
          {tr.t('Check')}
        </Button>
      </PageHeader>
      <NumberTiles
        tiles={[
          { value: n((list ?? []).filter((d) => d.state !== 'ok' && d.state !== 'restart').length), label: tr.t('newer drivers') },
          { value: n(chosen.length), label: tr.t('selected to install') },
          { value: n(s.problems.length), label: tr.t('devices with a problem') },
        ]}
        note={s.info.preview ? tr.t('Preview mode - you can check for drivers but not install them.') : tr.t('Drivers from Windows Update, like the optional updates in Settings. A restore point is created before installing.')}
      />
      <Card className="list-card">
        <div className="rows" onDoubleClick={() => void ui.api.openTarget('devices')}>
          {!sorted.length && !s.problems.length ? <EmptyText>{empty}</EmptyText> : null}
          {sorted.length ? <h3 className="group-title">{tr.t('Newer drivers from Windows Update')}</h3> : null}
          {!sorted.length && list !== null && s.problems.length ? <h3 className="group-title">{tr.t('Windows Update has no newer drivers for this PC')}</h3> : null}
          {sorted.map((d) => (
            <DriverRow key={d.id} driver={d} />
          ))}
          {s.problems.length ? <h3 className="group-title">{tr.t('Devices with a problem in Device Manager')}</h3> : null}
          {s.problems.map((p) => (
            <div key={p.id} className="driver-row problem" title={[p.name, tr.text(p.description), p.id].join('\n')}>
              <span className="badge badge-warn">
                <Icon name="warning" />
              </span>
              <span className="row-main">
                <strong>{p.name}</strong>
                <span className="small tone-warning">{tr.text(p.description)}</span>
              </span>
            </div>
          ))}
        </div>
      </Card>
      <Card className="plan-footer">
        <div className="footer-texts">
          <strong className="medium">{list === null ? tr.t('Not checked yet') : tr.t('Selected: {0}', String(chosen.length))}</strong>
          <span className="dim small">
            {list === null
              ? tr.t('Checking connects to Windows Update. Nothing is sent before that.')
              : tr.t('Up to {0} to download. With a graphics driver the screen may go blank for a moment. Some drivers take effect after a restart.', tr.size(chosen.reduce((a, d) => a + d.bytes, 0)))}
          </span>
        </div>
        <Button disabled={s.busy || s.info.preview || !chosen.some((d) => selectable(d, s.info.preview))} onClick={() => void drivers.install()}>
          {s.info.preview ? tr.t('Preview - installing disabled') : tr.t('Install selected...')}
        </Button>
      </Card>
    </div>
  )
}
