import icon from '../../../docs/icon.png'
import type { Translator } from '../../core/text/Translator'
import { windowFrame } from '../../platform/windowFrame'
import { TitleBar } from '../../shared/ui/titlebar'

/** The window's title bar (the shared one in src/shared/ui/titlebar); English before start-up. */
export function AppTitleBar({ tr }: { tr?: Translator }) {
  const t = (text: string) => (tr ? tr.t(text) : text)
  const labels = { minimize: t('Minimize'), maximize: t('Maximize'), restore: t('Restore'), close: t('Close') }
  return <TitleBar icon={<img src={icon} alt="" />} name="PC Cleaner" labels={labels} frame={windowFrame} />
}
