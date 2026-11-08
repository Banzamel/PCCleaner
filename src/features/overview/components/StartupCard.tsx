import { useServices, useUi } from '../../../app/AppContext'
import { STARTUP_ADVICE } from '../../../core/assistant/advicePrompts'
import { useT } from '../../../shared/i18n/useT'
import { Button } from '../../../shared/ui/Button'
import { Card } from '../../../shared/ui/Card'

/** How many programs start with Windows, and how many of them the AI says can be turned off. */
export function StartupCard() {
  const tr = useT()
  const { ui } = useServices()
  const startup = useUi((s) => s.startup)
  const ai = useUi((s) => s.ai)
  let text = tr.t('See which programs start when you sign in and slow down Windows startup.')
  if (startup) {
    const enabled = startup.filter((e) => e.isEnabled)
    text = tr.t('Programs starting at sign-in: {0}.', String(enabled.length))
    const optional = tr.lookup(STARTUP_ADVICE[1])
    const off = enabled.filter((e) => ai[e.id]?.includes(optional)).length
    if (off) text += ` ${tr.t('According to AI, {0} of them can be turned off.', String(off))}`
    else if (enabled.length) text += ` ${tr.t('Turning off the ones you do not need speeds up Windows startup.')}`
  }
  return (
    <Card className="startup-card">
      <h2>{tr.t('Windows startup')}</h2>
      <p className="dim">{text}</p>
      <Button look="link" onClick={() => ui.go('startup')}>
        {tr.t('Review startup apps')}
      </Button>
    </Card>
  )
}
