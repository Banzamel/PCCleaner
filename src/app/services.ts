import { AnalysisService } from '../features/analysis'
import { AssistantService } from '../features/assistant'
import { CleanService } from '../features/clean'
import { DriversService } from '../features/drivers'
import { ScanService } from '../features/overview'
import { PlanService } from '../features/plan'
import { SettingsService } from '../features/settings'
import { StartupService } from '../features/startup'
import { AutoCleanService } from './AutoCleanService'
import type { UiActions } from './UiActions'
import { UpdateService } from './UpdateService'

export interface Services {
  ui: UiActions
  scan: ScanService
  plan: PlanService
  clean: CleanService
  analysis: AnalysisService
  startup: StartupService
  drivers: DriversService
  assistant: AssistantService
  settings: SettingsService
  update: UpdateService
  autoClean: AutoCleanService
}

/** Creates every service with its dependencies (constructor injection, no module state). */
export function createServices(ui: UiActions): Services {
  const assistant = new AssistantService(ui)
  const startup = new StartupService(ui, assistant)
  const scan = new ScanService(ui, () => assistant.reset(), () => startup.load())
  // The cleaner stops the AI server (VRAM) and dead startup entries may be gone afterwards
  const clean = new CleanService(ui, async () => {
    await assistant.stop()
    await startup.load()
  })
  return {
    ui,
    scan,
    plan: new PlanService(ui, () => assistant.reset()),
    clean,
    analysis: new AnalysisService(ui, assistant),
    startup,
    drivers: new DriversService(ui),
    assistant,
    settings: new SettingsService(ui),
    update: new UpdateService(ui),
    autoClean: new AutoCleanService(ui, scan, clean),
  }
}
