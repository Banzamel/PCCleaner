// Disk analysis, startup apps, drivers, settings and app info returned by the Rust commands.

import type { Text } from './types'

export interface AnalysisItem {
  id: string
  kind: 'folder' | 'file'
  path: string
  bytes: number
  remainder: boolean
  files: number
  lastWrite: string | null
  hint: Text | null
}

export interface Analysis {
  letter: string
  size: number
  free: number
  scannedBytes: number
  folders: AnalysisItem[]
  files: AnalysisItem[]
  shadowCopies: number
  threshold: number
}

export interface StartupEntry {
  id: string
  kind: 'run' | 'folder' | 'task'
  name: string
  command: string
  file: string | null
  source: Text
  exists: boolean
  dead: boolean
  publisher: string
  protected: Text | null
  isEnabled: boolean
}

export interface DriverUpdate {
  id: string
  title: string
  name: string
  model: string
  provider: string
  class: string
  bytes: number
  version: string
  newDate: string | null
  currentVersion: string | null
  currentDate: string | null
  problem: Text | null
}

export interface ProblemDevice {
  name: string
  id: string
  code: number
  description: Text
}

export interface DriverCheck {
  drivers: DriverUpdate[]
  problems: ProblemDevice[]
}

export interface Config {
  language: string
  theme: string
  model: string
  assistant: boolean
  checkUpdates: boolean
  preview: boolean
  skipCategories: string[]
  scanLeftovers: boolean
  scanLargeFiles: boolean
  excludedFolders: string[]
  logDays: number
  closeToTray: boolean
  autoClean: string
  autoCleanLast: string
}

export interface AppInfo {
  version: string
  language: 'pl' | 'en' | 'es'
  model: string
  preview: boolean
  noAi: boolean
  noUpdateCheck: boolean
  autoClean: boolean
  otherUser: boolean
  portable: boolean
  logFile: string
  logDir: string
  systemDrive: string
  theme: string
  page: string
  drives: string[]
}

export interface AiStatus {
  installed: boolean
  models: string[]
  installerModel: string
}
