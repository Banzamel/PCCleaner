import type { UiActions } from '../../app/UiActions'
import type { AiStatus, CommandError, Config, Text } from '../../core/model/types'
import { tx } from '../../core/text/format'
import { pickFolder } from '../../platform/desktop'

/** The Settings page: saving settings.json, the logs, excluded folders and the AI install. */
export class SettingsService {
  constructor(private readonly ui: UiActions) {}

  /** Saves; the X button and the automatic cleaning task apply at once, the rest after a restart. */
  async save(config: Config): Promise<boolean> {
    try {
      const taskError = await this.ui.api.saveConfig(config)
      this.ui.store.set({ config })
      if (taskError) await this.ui.message(this.ui.tr.text(taskError), 'error')
      return true
    } catch (e) {
      await this.ui.message(this.ui.tr.text((e as CommandError).message), 'error')
      return false
    }
  }

  async saveAndRestart(config: Config): Promise<void> {
    if (await this.save(config)) await this.ui.api.restartApp()
  }

  async clearLogs(): Promise<Text> {
    const n = await this.ui.api.clearLogs()
    return tx('Logs deleted: {0}. The log of this run and the registry backups were kept.', String(n))
  }

  openLogs(): void {
    void this.ui.api.openTarget('logs')
  }

  pickFolder(): Promise<string | null> {
    return pickFolder(this.ui.tr.t('A folder the program should skip'))
  }

  aiStatus(): Promise<AiStatus> {
    return this.ui.api.aiStatus()
  }

  /** Downloads the AI engine and the model (several GB) into the program folder. */
  async installAi(model: string): Promise<boolean> {
    const ok = await this.ui.busy(async () => {
      this.ui.line(tx('Setting up the AI assistant (Ollama + model)...'), 'cyan')
      await this.ui.api.aiInstall(model)
      this.ui.line(tx('Done. The AI assistant is ready.'), 'green')
      return true
    })
    return ok === true
  }
}
