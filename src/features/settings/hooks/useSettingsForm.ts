import { useState } from 'react'

import { useServices, useUi } from '../../../app/AppContext'
import type { Config } from '../../../core/model/types'

/** A working copy of the settings; the page saves it with "Save" or "Save and restart". */
export function useSettingsForm() {
  const saved = useUi((s) => s.config)
  const info = useUi((s) => s.info)
  const { settings } = useServices()
  const [form, setForm] = useState<Config>(() => ({ ...saved, theme: saved.theme || 'auto', model: saved.model || info.model }))
  const [message, setMessage] = useState('')

  const set = <K extends keyof Config>(key: K, value: Config[K]) => setForm((f) => ({ ...f, [key]: value }))

  const scanChecked = (id: string): boolean =>
    id === '#leftovers' ? form.scanLeftovers : id === '#large' ? form.scanLargeFiles : !form.skipCategories.includes(id)

  const toggleScan = (id: string) => {
    if (id === '#leftovers') set('scanLeftovers', !form.scanLeftovers)
    else if (id === '#large') set('scanLargeFiles', !form.scanLargeFiles)
    else set('skipCategories', scanChecked(id) ? [...form.skipCategories, id] : form.skipCategories.filter((x) => x !== id))
  }

  /** The model chosen when the AI was installed is not written - a later install with another model still works. */
  const toSave = (installerModel: string): Config => ({
    ...form,
    model: form.model.trim() === installerModel ? '' : form.model.trim(),
    excludedFolders: form.excludedFolders.map((f) => f.trim()).filter(Boolean),
    logDays: form.logDays > 0 ? form.logDays : 0,
  })

  return { form, set, scanChecked, toggleScan, toSave, message, setMessage, settings, info }
}
