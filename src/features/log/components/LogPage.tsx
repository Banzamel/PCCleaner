import { useEffect, useRef } from 'react'

import { useUi } from '../../../app/AppContext'
import { useT } from '../../../shared/i18n/useT'
import { Card } from '../../../shared/ui/Card'
import { PageHeader } from '../../../shared/ui/PageHeader'

/** Everything the program does in this run; also written to the log file. */
export function LogPage() {
  const tr = useT()
  const log = useUi((s) => s.log)
  const file = useUi((s) => s.info.logFile)
  const end = useRef<HTMLDivElement>(null)
  useEffect(() => end.current?.scrollIntoView({ block: 'end' }), [log])
  return (
    <div className="log-page">
      <PageHeader title={tr.t('Log')} info={tr.t('Scan and cleaning progress. Full log: {0}', file)} />
      <Card className="log-card">
        <pre>
          {log.map((l, i) => (
            <span key={i} className={`log-${l.tone}`}>
              {tr.text(l.text)}
              {'\n'}
            </span>
          ))}
        </pre>
        <div ref={end} />
      </Card>
    </div>
  )
}
