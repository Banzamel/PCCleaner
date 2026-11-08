import type { Step } from '../core/plan/steps'
import type { PlanView } from '../core/plan/listing'
import type {
  Analysis,
  AppInfo,
  CleanResult,
  Config,
  DriveInfo,
  DriverUpdate,
  HistoryEntry,
  ProblemDevice,
  ScanReport,
  StartupEntry,
  Text,
  Tone,
} from '../core/model/types'
import type { Translator } from '../core/text/Translator'

export type PageId = 'overview' | 'plan' | 'analysis' | 'startup' | 'drivers' | 'chat' | 'log' | 'settings' | 'cleanup'

export interface LogLine {
  text: Text
  tone: Tone
}

export interface DriverRow extends DriverUpdate {
  selected: boolean
  state: string
  error: string
}

export interface ChatEntry {
  who: 'you' | 'ai' | 'note'
  text: string
}

export interface Dialog {
  title: string
  lines: Array<{ text: string; tone?: Tone; value?: string }>
  question?: string
  kind: 'confirm' | 'message'
  tone: 'info' | 'warning' | 'error'
}

/** The whole window state - one small store, changed only by the services. */
export interface UiState {
  info: AppInfo
  tr: Translator
  config: Config
  page: PageId
  busy: boolean
  status: Text
  drives: DriveInfo[]
  ticked: string[]
  report: ScanReport | null
  selected: Set<string>
  scanning: boolean
  stage: Text | null
  lastScan: string | null
  cleaning: boolean
  cleaned: boolean
  steps: Step[]
  cleanResult: CleanResult | null
  history: HistoryEntry[]
  log: LogLine[]
  plan: PlanView
  expanded: Set<string>
  analyses: Record<string, Analysis>
  analysisLetter: string
  startup: StartupEntry[] | null
  drivers: DriverRow[] | null
  problems: ProblemDevice[]
  driversNote: Text | null
  installingDrivers: boolean
  ai: Record<string, string>
  chat: ChatEntry[]
  update: { version: string } | null
  dialog: Dialog | null
}
