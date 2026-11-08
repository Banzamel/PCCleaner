import { useEffect } from 'react'

import { useServices } from './AppContext'
import { NAV } from './components/NavBar'

/** F5 scans, Ctrl+1..8 switch pages (not while cleaning or a dialog is open). */
export function useShortcuts(): void {
  const { ui, scan } = useServices()
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const s = ui.store.get()
      if (s.cleaning || s.dialog) return
      if (e.key === 'F5') {
        e.preventDefault()
        if (!s.busy) void scan.scan()
      } else if (e.ctrlKey && /^[1-8]$/.test(e.key)) {
        e.preventDefault()
        ui.go(NAV[Number(e.key) - 1].id)
      }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [ui, scan])
}
