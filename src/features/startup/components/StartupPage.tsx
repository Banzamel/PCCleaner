import { useEffect } from 'react'

import { useServices, useUi } from '../../../app/AppContext'
import { useT } from '../../../shared/i18n/useT'
import { Button } from '../../../shared/ui/Button'
import { Card } from '../../../shared/ui/Card'
import { EmptyText } from '../../../shared/ui/EmptyText'
import { NumberTiles } from '../../../shared/ui/NumberTiles'
import { PageHeader } from '../../../shared/ui/PageHeader'
import { StartupRow } from './StartupRow'

/** Programs that start at sign-in; the list loads on the first visit (a few seconds, no disk scan). */
export function StartupPage() {
  const tr = useT()
  const { startup } = useServices()
  const list = useUi((s) => s.startup)
  const busy = useUi((s) => s.busy)
  const scanning = useUi((s) => s.scanning)
  const preview = useUi((s) => s.info.preview)
  const noAi = useUi((s) => s.info.noAi)
  useEffect(() => {
    if (list === null && !scanning) void startup.refresh()
  }, [list, scanning, startup])
  const entries = list ?? []
  const enabled = entries.filter((e) => e.isEnabled).length
  const n = (v: number) => (list === null ? '-' : String(v))
  const sorted = [...entries].sort((a, b) => Number(!a.isEnabled) - Number(!b.isEnabled) || Number(!!a.protected) - Number(!!b.protected) || a.name.localeCompare(b.name))
  return (
    <div className="startup-page">
      <PageHeader
        title={tr.t('Startup apps')}
        info={tr.t(
          'Programs that start when you sign in. The switch works like in Task Manager: turning one off removes nothing. The bin removes the entry permanently, with a backup. Security software, drivers, updaters and parts of Windows are read-only.',
        )}
      >
        <Button look="outline" disabled={busy} onClick={() => void startup.refresh()}>
          {tr.t('Refresh')}
        </Button>
        <Button look="secondary" disabled={busy || noAi || !entries.length} onClick={() => void startup.describe()}>
          {tr.t('AI: what is it?')}
        </Button>
      </PageHeader>
      <NumberTiles
        tiles={[
          { value: n(enabled), label: tr.t('start at sign-in') },
          { value: n(entries.length - enabled), label: tr.t('turned off') },
          { value: n(entries.filter((e) => e.protected).length), label: tr.t('protected (read-only)') },
        ]}
        note={preview ? tr.t('Preview mode - the switches are read-only.') : tr.t('Changes take effect at the next sign-in. You can still start the programs by hand.')}
      />
      <Card className="list-card">
        <div className="rows">
          {!sorted.length ? <EmptyText>{list === null ? tr.t('The list loads when you open this page.') : tr.t('No programs that start at sign-in were found.')}</EmptyText> : null}
          {sorted.map((e) => (
            <StartupRow key={e.id} entry={e} />
          ))}
        </div>
      </Card>
    </div>
  )
}
