import { useServices, useUi } from '../../../app/AppContext'
import { useT } from '../../../shared/i18n/useT'
import { Button } from '../../../shared/ui/Button'
import { Icon } from '../../../shared/ui/Icon'
import { Segment } from '../../../shared/ui/Segment'

/** Search, the All / Selected / Opt-in filter and the selection buttons. */
export function PlanToolbar() {
  const tr = useT()
  const { plan } = useServices()
  const view = useUi((s) => s.plan)
  const locked = useUi((s) => !s.report || s.busy || s.cleaned)
  return (
    <div className="toolbar">
      <label className="search">
        <Icon name="search" />
        <input value={view.search} placeholder={tr.t('Search, e.g. Steam, cache, registry')} onChange={(e) => plan.setView({ search: e.target.value })} />
      </label>
      <Segment
        options={[
          { id: 'all', label: tr.t('All') },
          { id: 'selected', label: tr.t('Selected') },
          { id: 'optin', label: tr.t('Opt-in') },
        ]}
        value={view.filter}
        onChange={(f) => plan.setView({ filter: f })}
      />
      <span className="spacer" />
      <Button look="outline" disabled={locked} onClick={() => plan.clearSelection()}>
        {tr.t('Clear selection')}
      </Button>
      <Button look="outline" disabled={locked} onClick={() => plan.selectRecommended()}>
        {tr.t('Select recommended')}
      </Button>
    </div>
  )
}
