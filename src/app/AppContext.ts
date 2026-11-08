import { createContext, useContext, useSyncExternalStore } from 'react'

import type { Store } from '../shared/lib/Store'
import type { Services } from './services'
import type { UiState } from './state'

export interface AppContextValue {
  store: Store<UiState>
  services: Services
}

export const AppContext = createContext<AppContextValue | null>(null)

function useApp(): AppContextValue {
  const v = useContext(AppContext)
  if (!v) throw new Error('AppContext missing')
  return v
}

/** A slice of the window state; the component re-renders when it changes. */
export function useUi<T>(pick: (s: UiState) => T): T {
  const { store } = useApp()
  return useSyncExternalStore(store.subscribe, () => pick(store.get()))
}

export function useServices(): Services {
  return useApp().services
}
