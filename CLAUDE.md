# PC Cleaner

Windows 10/11 cleanup program: Tauri 2 (Rust) + React 19 + TypeScript (Vite). It scans, shows a plan with
sizes and cleans only after confirmation; an optional local AI assistant (Ollama) explains items.
Public repository `github.com/Banzamel/PCCleaner` - it knows nothing about any website or private registry.

## Code structure

The same rules in all of the owner's apps.

1. **File size:** aim for at most 150 lines, hard limit 200. Language files, JSON and data are exempt.
2. **One responsibility per file.** One React component per file. A file is named after its main export:
   `PascalCase.tsx` for components and classes, `camelCase.ts` for modules.
3. **Feature folders:**
   - `src/app/` - App, navigation, boot, the store and service wiring
   - `src/features/<feature>/` - `components/`, `hooks/`, `types.ts`, `index.ts` (overview, plan, clean,
     analysis, startup, drivers, assistant, log, settings)
   - `src/shared/{ui,hooks,i18n,lib}` - reusable pieces without feature knowledge
   - `src/core/<domain>/` - pure logic without React or Tauri (plan rules, summary, AI prompts and report)
   - `src/platform/` - the **only** place that imports `@tauri-apps/*` (typed commands, events, updater)
   - `src/lang/` - translations, `src/styles/` - CSS split per feature
4. **Classes for stateful services** (scan, cleaner, assistant client, settings, updater) with dependencies
   injected through the constructor; **pure functions** for calculations.
5. **TypeScript strict**; no `any` except wrapped at JSON/IPC boundaries.
6. Components mostly render; logic lives in hooks and services. One small global store, no mutable
   module globals.
7. Features import each other only through their `index.ts`.
8. **Rust:** `main.rs` is wiring only (60 lines at most). One module folder per area (`commands/`, `scan/`,
   `clean/`, `system/`, `storage/`...), small files, **one serde-serializable error type** (`AppError`),
   no `unwrap`/`expect` in command paths. Cleanup categories are **one trait + one file per category**
   (`src-tauri/core/src/scan/categories/`).
9. **Tests mirror the source tree:** Vitest for TypeScript (`test/`), `cargo test -p pccleaner-core` for Rust
   logic. Windows-only code sits behind `cfg(windows)` with the testable pure parts separated, so the core
   tests run on Linux.
10. Code and comments in **English**; comments explain **why**.

## Layout

| Path | Contents |
|---|---|
| `src-tauri/core/` | crate `pccleaner-core`: scanning, categories, leftovers, cleaning, startup apps, drivers, disk analysis, AI (Ollama), storage. No Tauri dependency |
| `src-tauri/src/` | the Tauri app: commands, events, tray, window state, elevation at start |
| `src-tauri/nsis/hooks.nsh` | installer hooks: migration from the PowerShell version, first-run language, uninstall cleanup |
| `src/` | the window (React) |
| `docker/build.sh` | Windows build in the `tauri-build` image (`npm run dist`) |

## Texts

User-facing texts are written in English in the code; `src/lang/pl.json` and `es.json` map the English text to
the translation (a missing one shows English). Rust returns `Text { key, args }` and the window translates it.
`test/lang/dictionaries.test.ts` fails when a text used in the code is missing from a dictionary.

## Safety rules (do not weaken)

- Deleting never follows junctions or symbolic links (`core/src/fs/`), excluded folders are never touched,
  protected folders (`fs/protect.rs`) are refused again at cleanup time.
- The window can only choose items by id from the last scan; it never sends a path to delete.
- Leftover folders and files go to the Recycle Bin; registry entries and tasks are backed up first
  (`.reg`/`.xml`) and a restore point is made.

## Release

Version source: `package.json` (`tauri.conf.json` reads it). Update `CHANGELOG.md`, then tag `v<version>`;
the workflow checks that the tag matches and publishes the setup, `.sig`, `latest.json` and the portable zip.
