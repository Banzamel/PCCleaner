import { useT } from '../../../shared/i18n/useT'
import { Button } from '../../../shared/ui/Button'
import type { useSettingsForm } from '../hooks/useSettingsForm'
import { SCAN_ITEMS } from '../scanItems'

type Form = ReturnType<typeof useSettingsForm>

/** What to scan, and folders that are never measured, listed or deleted. */
export function ScanScope({ f }: { f: Form }) {
  const tr = useT()
  const addFolder = async () => {
    const dir = await f.settings.pickFolder()
    if (dir) f.set('excludedFolders', [...f.form.excludedFolders.filter((x) => x.trim()), dir])
  }
  return (
    <>
      <h2 className="settings-heading">{tr.t('What to scan')}</h2>
      <p className="dim small">{tr.t('Unticked items are skipped by the scan and never reach the plan.')}</p>
      <div className="scan-checks">
        {SCAN_ITEMS.map((i) => (
          <label key={i.id} className="check">
            <input type="checkbox" checked={f.scanChecked(i.id)} onChange={() => f.toggleScan(i.id)} /> {tr.t(i.name)}
          </label>
        ))}
      </div>
      <div className="excluded">
        <strong>{tr.t('Excluded folders')}</strong>
        <span className="dim small">
          {tr.t('One path per line. The program does not measure them, does not list them in the plan and never deletes them - also when they lie inside a cache folder.')}
        </span>
        <div className="excluded-row">
          <textarea rows={5} value={f.form.excludedFolders.join('\n')} onChange={(e) => f.set('excludedFolders', e.target.value.split('\n'))} />
          <Button look="outline" onClick={() => void addFolder()}>
            {tr.t('Add a folder...')}
          </Button>
        </div>
      </div>
    </>
  )
}
