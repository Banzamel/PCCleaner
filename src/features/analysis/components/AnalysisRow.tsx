import { useServices, useUi } from '../../../app/AppContext'
import type { AnalysisItem } from '../../../core/model/types'
import { useT } from '../../../shared/i18n/useT'

/** Size, date, path; the second line is the AI description or the built-in hint. */
export function AnalysisRow({ item }: { item: AnalysisItem }) {
  const tr = useT()
  const { ui } = useServices()
  const ai = useUi((s) => s.ai[item.id])
  const path = item.remainder ? `${item.path}${` ${tr.t('(files outside the listed subfolders)')}`}` : item.path
  return (
    <div className="analysis-row" title={[item.path, item.hint ? tr.text(item.hint) : ''].filter(Boolean).join('\n')} onDoubleClick={() => void ui.api.openTarget('reveal', item.path)}>
      <strong className="num">{tr.size(item.bytes)}</strong>
      <span className="dim small">{item.lastWrite ?? ''}</span>
      <span className="row-main">
        <span>{path}</span>
        {ai ? (
          <span className="small tone-ai">
            {tr.t('AI (may be wrong)')}: {ai}
          </span>
        ) : item.hint ? (
          <span className="small dim">{tr.text(item.hint)}</span>
        ) : null}
      </span>
    </div>
  )
}
