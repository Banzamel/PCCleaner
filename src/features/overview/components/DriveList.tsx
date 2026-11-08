import { useServices, useUi } from '../../../app/AppContext'
import { useT } from '../../../shared/i18n/useT'
import { Bar } from '../../../shared/ui/Bar'
import { Card } from '../../../shared/ui/Card'
import { CheckBox } from '../../../shared/ui/CheckBox'

/** Drives to scan with how full they are. The system drive is always scanned. */
export function DriveList() {
  const tr = useT()
  const { scan } = useServices()
  const drives = useUi((s) => s.drives)
  const ticked = useUi((s) => s.ticked)
  const busy = useUi((s) => s.busy)
  return (
    <Card className="drive-card">
      <h2>{tr.t('Drives to scan')}</h2>
      <ul className="drive-list">
        {drives.map((d) => {
          const used = d.size > 0 ? 1 - d.free / d.size : 0
          const name = `${d.letter}  ${d.label || tr.t('no label')}${d.system ? ` ${tr.t('(system)')}` : ''}`
          return (
            <li key={d.letter} className={d.system ? '' : 'clickable'} onClick={() => scan.toggleDrive(d.letter)}>
              <CheckBox checked={ticked.includes(d.letter)} disabled={d.system || busy} label={name} onToggle={() => scan.toggleDrive(d.letter)} />
              <div className="drive-text">
                <strong>{name}</strong>
                <span className="dim small">{tr.t('{0} free of {1}', tr.size(d.free), tr.size(d.size))}</span>
                <Bar value={used} tone={used > 0.9 ? 'red' : used > 0.75 ? 'warning' : 'accent'} size="thin" />
              </div>
            </li>
          )
        })}
      </ul>
    </Card>
  )
}
