import { useEffect, useState } from 'react'

import type { AiStatus } from '../../../core/model/types'
import { useT } from '../../../shared/i18n/useT'
import { Button } from '../../../shared/ui/Button'
import { Card } from '../../../shared/ui/Card'
import { PageHeader } from '../../../shared/ui/PageHeader'
import { useSettingsForm } from '../hooks/useSettingsForm'
import { AiSetup } from './AiSetup'
import { GeneralSettings } from './GeneralSettings'
import { ScanScope } from './ScanScope'

/** Settings, saved to %LOCALAPPDATA%\PCCleaner\settings.json; command-line parameters win over them. */
export function SettingsPage() {
  const tr = useT()
  const f = useSettingsForm()
  const [ai, setAi] = useState<AiStatus | null>(null)
  const reloadAi = () => void f.settings.aiStatus().then(setAi).catch(() => setAi(null))
  useEffect(reloadAi, [f.settings])
  const installer = ai?.installerModel ?? f.info.model
  const save = async () => {
    if (await f.settings.save(f.toSave(installer))) f.setMessage(tr.t('Saved. Changes take effect after a restart.'))
  }
  return (
    <div className="settings-page">
      <PageHeader title={tr.t('Settings')} info={tr.t('Changes take effect after the program restarts.')} />
      <Card className="settings-card">
        <GeneralSettings f={f} models={ai?.models ?? []} />
        <AiSetup status={ai} onInstalled={reloadAi} />
        <ScanScope f={f} />
      </Card>
      <div className="settings-bottom">
        <span className="dim">{f.message}</span>
        <Button look="secondary" onClick={() => void save()}>
          {tr.t('Save')}
        </Button>
        <Button onClick={() => void f.settings.saveAndRestart(f.toSave(installer))}>{tr.t('Save and restart')}</Button>
      </div>
    </div>
  )
}
