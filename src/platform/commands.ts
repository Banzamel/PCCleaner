// The only place that calls Rust commands. Every function is typed; errors arrive as CommandError.

import { invoke } from '@tauri-apps/api/core'

import type { ChatMessage } from '../core/assistant/Conversation'
import type {
  AiStatus,
  Analysis,
  AppInfo,
  CleanResult,
  CommandError,
  Config,
  DriveInfo,
  DriverCheck,
  HistoryEntry,
  ScanReport,
  StartupEntry,
  Text,
} from '../core/model/types'

/** Turns anything thrown by invoke into a CommandError (the Rust AppError, or a plain message). */
export function asCommandError(e: unknown): CommandError {
  if (typeof e === 'object' && e !== null && 'message' in e) {
    const m = (e as { message: unknown; detail?: unknown }).message
    if (typeof m === 'object' && m !== null && 'key' in m) return e as CommandError
  }
  const detail = typeof e === 'string' ? e : e instanceof Error ? e.message : JSON.stringify(e)
  return { message: { key: 'An error occurred: {0}', args: [detail] }, detail }
}

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(cmd, args)
  } catch (e) {
    throw asCommandError(e)
  }
}

export const commands = {
  appInfo: () => call<AppInfo>('app_info'),
  listDrives: () => call<DriveInfo[]>('list_drives'),
  scan: (drives: string[]) => call<ScanReport>('scan', { drives }),
  clean: (categories: string[], leftovers: string[]) => call<CleanResult>('clean', { categories, leftovers }),
  history: () => call<HistoryEntry[]>('cleanup_history'),
  analyse: (letter: string) => call<Analysis>('analyse', { letter }),
  startupList: () => call<StartupEntry[]>('startup_list'),
  startupSet: (id: string, enable: boolean) => call<void>('startup_set', { id, enable }),
  startupRemove: (id: string) => call<Text>('startup_remove', { id }),
  driversCheck: () => call<DriverCheck>('drivers_check'),
  driversInstall: (ids: string[]) => call<Text>('drivers_install', { ids }),
  aiStatus: () => call<AiStatus>('ai_status'),
  aiPrepare: (model: string) => call<void>('ai_prepare', { model }),
  aiChat: (model: string, messages: ChatMessage[], format?: object) => call<string>('ai_chat', { model, messages, format: format ?? null }),
  aiStop: () => call<void>('ai_stop'),
  aiInstall: (model: string) => call<void>('ai_install', { model }),
  getConfig: () => call<Config>('get_config'),
  saveConfig: (config: Config) => call<Text | null>('save_config', { config }),
  autoCleanDone: () => call<void>('auto_clean_done'),
  clearLogs: () => call<number>('clear_logs'),
  openTarget: (kind: 'reveal' | 'logs' | 'log' | 'devices' | 'windows-update' | 'url', path?: string) => call<void>('open_target', { kind, path: path ?? null }),
  savePage: (page: string) => call<void>('save_page', { page }),
  setProgress: (state: 'none' | 'indeterminate' | 'normal' | 'error', value = 0) => call<void>('set_progress', { state, value: Math.round(value) }),
  exitApp: () => call<void>('exit_app'),
  restartApp: () => call<void>('restart_app'),
  hideToTray: (quiet: boolean) => call<void>('hide_to_tray', { quiet }),
  showWindow: () => call<void>('show_window'),
  windowVisible: () => call<boolean>('window_visible'),
  trayTexts: (open: string, exit: string, tooltip: string) => call<void>('tray_texts', { open, exit, tooltip }),
  notify: (title: string, body: string) => call<void>('notify', { title, body }),
}

export type Commands = typeof commands
