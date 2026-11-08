import { useServices, useUi } from '../../../app/AppContext'
import { useT } from '../../../shared/i18n/useT'
import { Button } from '../../../shared/ui/Button'
import { Card } from '../../../shared/ui/Card'
import { EmptyText } from '../../../shared/ui/EmptyText'
import { PageHeader } from '../../../shared/ui/PageHeader'
import { AnalysisRow } from './AnalysisRow'

/** "What takes up space": read-only, nothing here is deleted. */
export function AnalysisPage() {
  const tr = useT()
  const { analysis } = useServices()
  const s = useUi((st) => st)
  const a = s.analyses[s.analysisLetter]
  const drives = s.drives.filter((d) => s.ticked.includes(d.letter))
  let status = ''
  if (a) {
    status = tr.t('Drive {0}: {1} used of {2}, {3} free. Read-only - nothing here is deleted. Double-click opens the folder.', a.letter, tr.size(a.size - a.free), tr.size(a.size), tr.size(a.free))
    if (a.shadowCopies > 0) status += ` ${tr.t('Restore points: {0}.', tr.size(a.shadowCopies))}`
    const unreadable = a.size - a.free - a.scannedBytes - a.shadowCopies
    if (unreadable > 2 * 1024 ** 3) status += ` ${tr.t('Not readable by the scanner: about {0}.', tr.size(unreadable))}`
  }
  return (
    <div className="analysis-page">
      <PageHeader
        title={tr.t('What takes up space')}
        info={tr.t('A fast scan of the whole drive (usually 30 s - 3 min). Shows the most specific large folders and the largest files. Read-only - nothing here is deleted.')}
        status={status}
      />
      <Card className="toolbar-card">
        <label className="field-label">{tr.t('Drive')}</label>
        <select value={s.analysisLetter} onChange={(e) => analysis.pick(e.target.value)} disabled={s.busy}>
          {drives.map((d) => (
            <option key={d.letter} value={d.letter.slice(0, 1)}>
              {tr.t('{0} {1} - {2} free of {3}', d.letter, d.label || tr.t('no label'), tr.size(d.free), tr.size(d.size))}
            </option>
          ))}
        </select>
        <Button disabled={s.busy || !s.report} onClick={() => void analysis.analyse()}>
          {tr.t('Analyse')}
        </Button>
        <Button look="secondary" disabled={s.busy || !a || s.info.noAi} onClick={() => void analysis.describe()}>
          {tr.t('AI: describe and advise')}
        </Button>
      </Card>
      <Card className="list-card">
        {!a ? (
          <EmptyText>{tr.t('Pick a drive and click "Analyse". The list works after a scan from Overview.')}</EmptyText>
        ) : !a.folders.length && !a.files.length ? (
          <EmptyText>{tr.t('There are no large folders or files on this drive.')}</EmptyText>
        ) : (
          <div className="rows">
            {a.folders.length ? <h3 className="group-title">{tr.t('Largest folders')}</h3> : null}
            {a.folders.map((i) => (
              <AnalysisRow key={i.id} item={i} />
            ))}
            {a.files.length ? <h3 className="group-title">{tr.t('Largest files')}</h3> : null}
            {a.files.map((i) => (
              <AnalysisRow key={i.id} item={i} />
            ))}
          </div>
        )}
      </Card>
    </div>
  )
}
