import type { Category, Leftover, ProfileScan } from '../src/core/model/types'

export function category(p: Partial<Category> & { id: string }): Category {
  return {
    name: { key: p.id },
    description: { key: 'd' },
    kind: 'folder',
    optIn: false,
    bytes: 0,
    files: 0,
    locked: 0,
    note: null,
    blocker: null,
    hasProcesses: false,
    measurable: true,
    ...p,
  }
}

export function leftover(p: Partial<Leftover> & { id: string }): Leftover {
  return {
    kind: 'folder',
    path: `C:\\X\\${p.id}`,
    name: p.id,
    bytes: 0,
    lastWrite: null,
    confidence: 'high',
    reason: { key: 'No installed program with this name.' },
    game: false,
    largeFile: false,
    subkeys: [],
    valueName: null,
    folder: null,
    ...p,
  }
}

export const emptyProfile: ProfileScan = { profile: [], appdata: [], programs: [], hiberfil: null, windowsOld: false, oldSetupBt: false, drives: [] }

export const GB = 1024 ** 3
