// Window theme, folder picker and the GitHub updater.

import { getCurrentWindow } from '@tauri-apps/api/window'
import { open } from '@tauri-apps/plugin-dialog'
import { check, type Update } from '@tauri-apps/plugin-updater'

/** Title bar theme; null follows Windows. */
export async function setWindowTheme(theme: 'light' | 'dark' | null): Promise<void> {
  try {
    await getCurrentWindow().setTheme(theme)
  } catch {
    // An older WebView without theme support keeps the default title bar
  }
}

export async function pickFolder(title: string): Promise<string | null> {
  const r = await open({ directory: true, multiple: false, title })
  return typeof r === 'string' ? r : null
}

export interface AvailableUpdate {
  version: string
  install: () => Promise<void>
}

/** One request to GitHub Releases (latest.json); null when up to date or offline. */
export async function findUpdate(): Promise<AvailableUpdate | null> {
  const u: Update | null = await check({ timeout: 10000 })
  if (!u) return null
  return { version: u.version, install: () => u.downloadAndInstall() }
}
