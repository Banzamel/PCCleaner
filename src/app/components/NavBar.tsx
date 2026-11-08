import { useT } from '../../shared/i18n/useT'
import { Button } from '../../shared/ui/Button'
import { Icon, type GlyphName } from '../../shared/ui/Icon'
import { useServices, useUi } from '../AppContext'
import type { PageId } from '../state'

export const NAV: Array<{ id: PageId; label: string; icon: GlyphName }> = [
  { id: 'overview', label: 'Overview', icon: 'overview' },
  { id: 'plan', label: 'Cleanup plan', icon: 'plan' },
  { id: 'analysis', label: 'What takes up space', icon: 'analysis' },
  { id: 'startup', label: 'Startup apps', icon: 'startup' },
  { id: 'drivers', label: 'Drivers', icon: 'drivers' },
  { id: 'chat', label: 'AI assistant', icon: 'chat' },
  { id: 'log', label: 'Log', icon: 'log' },
  { id: 'settings', label: 'Settings', icon: 'settings' },
]

/** Dark navigation bar (the logo is in the title bar): pages (Ctrl+1..8), assistant status, preview mode, update. */
export function NavBar() {
  const tr = useT()
  const { ui, update } = useServices()
  const page = useUi((s) => s.page)
  const cleaning = useUi((s) => s.cleaning)
  const info = useUi((s) => s.info)
  const available = useUi((s) => s.update)
  const busy = useUi((s) => s.busy)
  return (
    <nav className="nav">
      {cleaning ? (
        <p className="nav-info">{tr.t('Cleaning in progress. Navigation comes back when it finishes.')}</p>
      ) : (
        <ul className="nav-list">
          {NAV.map((n, i) => (
            <li key={n.id}>
              <button type="button" className={page === n.id ? 'on' : ''} title={`Ctrl+${i + 1}`} onClick={() => ui.go(n.id)}>
                <Icon name={n.icon} />
                <span>{tr.t(n.label)}</span>
              </button>
            </li>
          ))}
        </ul>
      )}
      <div className="nav-bottom">
        {info.preview ? <p className="nav-preview">{tr.t('PREVIEW MODE - nothing will be deleted')}</p> : null}
        <p className="nav-ai">{info.noAi ? tr.t('AI assistant: off') : tr.t('AI assistant: on demand\n{0}', info.model)}</p>
        {available ? (
          <Button look="green" disabled={busy} onClick={() => void update.install()}>
            {tr.t('Install version {0}', available.version)}
          </Button>
        ) : null}
      </div>
    </nav>
  )
}
