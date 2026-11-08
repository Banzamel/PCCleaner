// Scan, plan and cleanup results returned by the Rust commands.

import type { Text } from './types'

export type CategoryKind = 'folder' | 'recyclebin' | 'thumbnails' | 'dns' | 'winold' | 'dism' | 'driverstore' | 'docker' | 'dockerall'

export interface Category {
  id: string
  name: Text
  description: Text
  kind: CategoryKind
  optIn: boolean
  bytes: number
  files: number
  locked: number
  note: Text | null
  blocker: Text | null
  hasProcesses: boolean
  measurable: boolean
}

export type LeftoverKind = 'folder' | 'file' | 'registry' | 'regvalue' | 'task'
export type Confidence = 'high' | 'medium' | 'low'

export interface Leftover {
  id: string
  kind: LeftoverKind
  path: string
  name: string
  bytes: number
  lastWrite: string | null
  confidence: Confidence
  reason: Text
  game: boolean
  largeFile: boolean
  subkeys: string[]
  valueName: string | null
  folder: string | null
}

export interface DriveInfo {
  letter: string
  label: string
  size: number
  free: number
  system: boolean
}

export interface NamedSize {
  name: string
  bytes: number
  installed?: string
}

export interface ProfileScan {
  profile: NamedSize[]
  appdata: NamedSize[]
  programs: NamedSize[]
  hiberfil: number | null
  windowsOld: boolean
  oldSetupBt: boolean
  drives: DriveInfo[]
}

export interface ScanReport {
  categories: Category[]
  leftovers: Leftover[]
  profile: ProfileScan
  freeBefore: number
  scanErrors: number
  drives: string[]
}

export interface CleanResult {
  total: number
  durationMs: number
  freeBefore: number
  freeAfter: number
  backups: string | null
  backupCount: number
  restorePoint: Text | null
  locked: number
  inRecycleBin: number
}

export interface HistoryEntry {
  date: string
  drive: string
  total: number
  freeBefore: number
  freeAfter: number
}
