import { useT } from '../../../shared/i18n/useT'
import { Button } from '../../../shared/ui/Button'
import type { useSettingsForm } from '../hooks/useSettingsForm'
import { SettingsRow } from './SettingsRow'

type Form = ReturnType<typeof useSettingsForm>

/** Language, theme, assistant, model, updates, the X button, automatic cleaning, preview, logs. */
export function GeneralSettings({ f, models }: { f: Form; models: string[] }) {
  const tr = useT()
  const { form, set } = f
  const select = (value: string, options: Array<[string, string]>, onChange: (v: string) => void, disabled = false) => (
    <select value={value} disabled={disabled} onChange={(e) => onChange(e.target.value)}>
      {options.map(([v, label]) => (
        <option key={v} value={v}>
          {label}
        </option>
      ))}
    </select>
  )
  const check = (on: boolean, label: string, onChange: (v: boolean) => void) => (
    <label className="check">
      <input type="checkbox" checked={on} onChange={(e) => onChange(e.target.checked)} /> {label}
    </label>
  )
  return (
    <>
      <SettingsRow title={tr.t('Language')} description={tr.t('The language of the window and of the AI assistant.')}>
        {select(form.language, [['auto', tr.t('Same as Windows')], ['pl', 'Polski'], ['en', 'English'], ['es', 'Español']], (v) => set('language', v))}
      </SettingsRow>
      <SettingsRow title={tr.t('Theme')} description={tr.t('Light or dark look of the window.')}>
        {select(form.theme, [['auto', tr.t('Same as Windows')], ['light', tr.t('Light')], ['dark', tr.t('Dark')]], (v) => set('theme', v))}
      </SettingsRow>
      <SettingsRow title={tr.t('AI assistant')} description={tr.t('The local model describes items and answers in the chat. When off it uses no memory.')}>
        {check(form.assistant, tr.t('On'), (v) => set('assistant', v))}
      </SettingsRow>
      <SettingsRow title={tr.t('AI model')} description={tr.t('A model from the ai\\models folder. Get another one with "Install" below.')}>
        <input list="local-models" value={form.model} onChange={(e) => set('model', e.target.value)} />
        <datalist id="local-models">
          {models.map((m) => (
            <option key={m} value={m} />
          ))}
        </datalist>
      </SettingsRow>
      <SettingsRow title={tr.t('Updates')} description={tr.t('One request to GitHub at startup, with no data about this computer.')}>
        {check(form.checkUpdates, tr.t('Check at startup'), (v) => set('checkUpdates', v))}
      </SettingsRow>
      <SettingsRow title={tr.t('The X button')} description={tr.t('In the tray it keeps running - exit it from the icon menu.')}>
        {select(form.closeToTray ? '1' : '0', [['0', tr.t('Closes the program')], ['1', tr.t('Hides the window to the tray')]], (v) => set('closeToTray', v === '1'))}
      </SettingsRow>
      <SettingsRow title={tr.t('Automatic cleaning')} description={tr.t('After sign-in cleans the recommended categories (no leftovers).')}>
        {select(
          form.autoClean,
          [['off', tr.t('Never')], ['logon', tr.t('At every sign-in')], ['daily', tr.t('Once a day, at sign-in')], ['weekly', tr.t('Once a week, at sign-in')]],
          (v) => set('autoClean', v),
          // The task runs as the signed-in account - with another administrator's password it would clean the wrong profile
          f.info.otherUser,
        )}
      </SettingsRow>
      <SettingsRow title={tr.t('Preview mode')} description={tr.t('Scan and plan without deleting anything - safe for showing the program to someone.')}>
        {check(form.preview, tr.t('Always in preview mode'), (v) => set('preview', v))}
      </SettingsRow>
      <SettingsRow title={tr.t('Logs')} description={tr.t('The log of every run and registry backups: {0}', f.info.logDir)}>
        <div className="row-buttons">
          <Button look="outline" onClick={() => f.settings.openLogs()}>
            {tr.t('Open the folder')}
          </Button>
          <Button look="outline" onClick={() => void f.settings.clearLogs().then((t) => f.setMessage(tr.text(t)))}>
            {tr.t('Clear the logs')}
          </Button>
        </div>
      </SettingsRow>
      <SettingsRow title={tr.t('Old logs')} description={tr.t('At startup deletes logs older than 30 days. Registry backups are kept.')}>
        {check(form.logDays > 0, tr.t('Delete ones older than 30 days'), (v) => set('logDays', v ? 30 : 0))}
      </SettingsRow>
    </>
  )
}
