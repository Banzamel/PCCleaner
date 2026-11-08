import { useEffect, useState } from 'react'

import { useServices, useUi } from '../../../app/AppContext'
import type { AiStatus } from '../../../core/model/types'
import { on } from '../../../platform/events'
import { useT } from '../../../shared/i18n/useT'
import { Bar } from '../../../shared/ui/Bar'
import { Button } from '../../../shared/ui/Button'
import { SettingsRow } from './SettingsRow'

const MODELS: Array<[string, string]> = [
  ['qwen3:4b-instruct', 'qwen3:4b-instruct - recommended, fast, fits GPUs with 6 GB VRAM (2.5 GB download)'],
  ['qwen3:8b', 'qwen3:8b - smarter, needs about 8 GB VRAM, slower on smaller GPUs (5.2 GB download)'],
]

/** Installs the AI engine (Ollama) and a model into the program folder - only when the user asks. */
export function AiSetup({ status, onInstalled }: { status: AiStatus | null; onInstalled: () => void }) {
  const tr = useT()
  const { settings } = useServices()
  const busy = useUi((s) => s.busy)
  const [model, setModel] = useState(MODELS[0][0])
  const [progress, setProgress] = useState<{ text: string; fraction: number } | null>(null)
  useEffect(() => on('pcc://ai-install', (p) => setProgress({ text: tr.text(p.text), fraction: p.fraction })), [tr])
  const install = async () => {
    const ok = await settings.installAi(model)
    setProgress(null)
    if (ok) onInstalled()
  }
  const installed = status?.installed ? tr.t('Installed: {0}', status.models.join(', ') || '-') : tr.t('The AI assistant is not installed.')
  return (
    <SettingsRow
      title={tr.t('Install the AI assistant')}
      description={tr.t('The AI engine (Ollama, 1.4 GB) and the model are stored in the program folder and removed on uninstall. The assistant only explains leftovers and answers questions - cleaning works without it.')}
    >
      <div className="ai-setup">
        <span className="small dim">{installed}</span>
        <select value={model} onChange={(e) => setModel(e.target.value)} disabled={busy}>
          {MODELS.map(([v, label]) => (
            <option key={v} value={v}>
              {tr.t(label)}
            </option>
          ))}
        </select>
        <Button look="outline" disabled={busy} onClick={() => void install()}>
          {tr.t('Install')}
        </Button>
        {progress ? (
          <>
            <span className="small">{progress.text}</span>
            <Bar value={progress.fraction} size="thin" />
          </>
        ) : null}
      </div>
    </SettingsRow>
  )
}
