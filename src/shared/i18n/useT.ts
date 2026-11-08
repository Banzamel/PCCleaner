import { useUi } from '../../app/AppContext'
import type { Translator } from '../../core/text/Translator'

/** The translator of the current language (t, text, size). */
export function useT(): Translator {
  return useUi((s) => s.tr)
}
