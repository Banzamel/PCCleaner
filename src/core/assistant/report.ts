import type { Analysis, Category, Leftover, ProfileScan, StartupEntry } from '../model/types'
import { categorySize, leftoverLabel, leftoverSize } from '../plan/describe'
import { selectedTotal } from '../plan/rules'
import type { Translator } from '../text/Translator'

export interface ReportInput {
  categories: Category[]
  leftovers: Leftover[]
  selected: Set<string>
  profile: ProfileScan
  analyses: Analysis[]
  startup: StartupEntry[]
  /** AI descriptions by item id (leftovers, analysis items, startup entries). */
  ai: Record<string, string>
}

/** Limit in characters (about 2.5 characters per token) so a small model reads it in full. */
export const CHAT_REPORT_LIMIT = 14000

const cut = (t: string | undefined, n: number): string | undefined => (!t || t.length <= n ? t : `${t.slice(0, n - 1)}…`)

/**
 * The report for the chat, trimmed to fit the model's context window: the full report after a big
 * scan overflowed it and the model lost its instructions. The largest items stay, the rest is counted.
 */
export function chatReport(r: ReportInput, tr: Translator, limit = CHAT_REPORT_LIMIT): string {
  const left = [...r.leftovers].sort((a, b) => b.bytes - a.bytes)
  let json = ''
  for (const n of [60, 40, 25, 15, 8, 4]) {
    const shown = left.slice(0, n)
    const rest = left.slice(n)
    const p = r.profile
    const report = {
      summary: {
        selected_total: tr.size(selectedTotal(r.categories, r.leftovers, r.selected)),
        leftovers_total: left.length,
        leftovers_selected: left.filter((l) => r.selected.has(l.id)).length,
        leftovers_hidden: rest.length,
        hidden_total: tr.size(rest.reduce((s, l) => s + l.bytes, 0)),
      },
      drives: p.drives.map((d) => ({ drive: d.letter, free: tr.size(d.free), total: tr.size(d.size) })),
      system: { hiberfil: p.hiberfil === null ? 'none (hibernation off)' : tr.size(p.hiberfil), windows_old: p.windowsOld, old_setup_bt: p.oldSetupBt },
      categories: r.categories.map((c) => ({
        name: tr.text(c.name),
        size: tr.text(categorySize(c)),
        selected: r.selected.has(c.id),
        ...(c.optIn ? { opt_in: true } : {}),
        ...(c.blocker ? { blocker: cut(tr.text(c.blocker), 80) } : {}),
      })),
      leftovers: shown.map((l) => ({
        id: l.id,
        type: l.kind,
        path: cut(tr.text(leftoverLabel(l)), 110),
        size: tr.text(leftoverSize(l)),
        selected: r.selected.has(l.id),
        confidence: l.confidence,
        reason: cut(tr.text(l.reason), 90),
        ...(l.lastWrite ? { last_modified: l.lastWrite } : {}),
        ...(r.ai[l.id] ? { what_it_is: cut(r.ai[l.id], 80) } : {}),
      })),
      largest_programs: p.programs.slice(0, Math.min(n, 12)).map((x) => ({ name: x.name, size: tr.size(x.bytes), installed: x.installed })),
      largest_folders: [...p.profile, ...p.appdata].slice(0, Math.min(n, 12)).map((x) => ({ folder: x.name, size: tr.size(x.bytes) })),
      disk_analysis: r.analyses.map((a) => ({
        drive: `${a.letter}:`,
        items: [...a.folders, ...a.files]
          .sort((x, y) => y.bytes - x.bytes)
          .slice(0, Math.min(n, 15))
          .map((i) => ({
            path: cut(i.path, 110),
            size: tr.size(i.bytes),
            ...(i.hint ? { hint: cut(tr.text(i.hint), 80) } : {}),
            ...(r.ai[i.id] ? { what_it_is: cut(r.ai[i.id], 80) } : {}),
          })),
      })),
      startup: r.startup.slice(0, Math.min(n, 30)).map((s) => ({
        name: s.name,
        enabled: s.isEnabled,
        ...(s.publisher ? { publisher: cut(s.publisher, 40) } : {}),
        ...(r.ai[s.id] ? { what_it_is: cut(r.ai[s.id], 60) } : {}),
      })),
    }
    json = JSON.stringify(report)
    if (json.length <= limit) return json
  }
  return json
}
