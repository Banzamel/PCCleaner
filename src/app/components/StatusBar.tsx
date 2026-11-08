import { useT } from '../../shared/i18n/useT'
import { useUi } from '../AppContext'

/** Status line under the pages, with a running indicator while something works. */
export function StatusBar() {
  const tr = useT()
  const status = useUi((s) => s.status)
  const busy = useUi((s) => s.busy)
  return (
    <footer className="status-bar">
      <span className="status-text">{tr.text(status)}</span>
      {busy ? <span className="marquee" aria-hidden="true" /> : null}
    </footer>
  )
}
