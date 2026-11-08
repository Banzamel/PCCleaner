<p align="center">
  <img src="docs/icon.png" width="96" alt="PC Cleaner icon">
</p>

# PC Cleaner

**Smart Windows cleanup with a local AI assistant.** PC Cleaner finds junk files, caches and leftovers of uninstalled
programs and games, shows you a plan with exact sizes, and cleans only after you confirm. An optional local AI model
explains what the leftovers were and answers your questions. The AI engine ([Ollama](https://ollama.com)) is installed
into the program folder on request, so everything runs on your machine. The only network requests are an update check
against GitHub Releases at start (turn it off in *Settings* or with `--no-update-check`), the driver check on Windows
Update, which runs only when you click **Check** on the *Drivers* page, and the AI download (the Ollama engine from
GitHub and the model from the Ollama library), which runs only when you click **Install** in *Settings*.

> The interface is in **English, Polish or Spanish**, chosen in the installer, in *Settings* or from the Windows display
> language (`--language en` / `pl` / `es` to override).

---

## Features

- **Scan first, clean later.** You tick the drives to scan and click *Scan*. The plan appears as a checklist with sizes
  and descriptions. You change it, see a summary with the space to be freed, and confirm. The default answer is *no*.
- **Whitelist-based cleanup** of things that rebuild themselves: temp files, Windows Update cache, crash dumps,
  browser caches (Chrome, Edge, Brave, Firefox: all profiles; passwords, bookmarks and sessions are kept),
  npm/pip/yarn/uv/NuGet/Composer/Go/Gradle/Cargo/Bun caches, GPU shader caches, thumbnails, DNS cache, old NVIDIA/AMD
  driver installers, driver crash reports (`LiveKernelReports`), app caches (Discord, Slack, Teams, VS Code, Spotify,
  Steam, Epic Games Launcher, Battle.net; each app is skipped while it is running), JetBrains IDE caches and indexes
  (opt-in), and optionally DISM, older driver versions in the DriverStore, the previous Windows version
  (`Windows.old`, removed through Windows Disk Cleanup) and the Docker build cache.
- **What takes up space.** A fast scan of any drive lists the most specific large folders and the largest files, with
  built-in hints (WinSxS, hiberfil, WSL disks, Steam games, `node_modules`…). The AI can describe each item and suggest
  what to do. Its advice comes from a fixed list of safe options, and for `Windows`, `ProgramData` and `Program Files`
  the advice is set by the program, not by the model. This view is read-only: nothing from it is deleted.
- **Startup apps.** A list of programs that start when you sign in (Run registry keys, Startup folders, Task
  Scheduler tasks with a sign-in or boot trigger) with publisher and command. Turning one off uses the same switch as
  Task Manager, so nothing is removed and it can be turned back on. Security software, drivers, hardware panels,
  updaters and parts of Windows are read-only. The AI can describe each program and suggest whether it needs to start
  with Windows.
- **Automatic cleaning (optional, off by default).** In *Settings* you can let PC Cleaner clean at every sign-in, once
  a day or once a week. A Task Scheduler task of your account starts it with administrator rights about two minutes
  after you sign in; it runs in the notification area, cleans only the categories the plan recommends (never leftovers
  or large files) and shows the freed space in a notification. Open it from the tray icon; otherwise the program
  closes itself. Preview mode only reports what would be cleaned. Turning the option off removes the task.
- **Driver updates.** The *Drivers* page looks for newer drivers for this PC's devices on Windows Update (the same
  signed drivers as the optional updates in Settings), shows the current and new version of each, and installs the
  ones you tick after a System Restore point. It also lists devices with a problem in Device Manager.
- **Leftovers of uninstalled software:**
  - folders in `AppData`, `LocalLow`, `ProgramData` and `Program Files` that match no installed program,
  - Steam games removed without cleanup (folders without a manifest, shader caches, Workshop content) and Epic Games leftovers,
  - dead entries in *Apps & features* and orphaned registry keys, listed with their subkeys
    (e.g. `Klei` → *contains: Oxygen Not Included*),
  - dead startup entries, shortcuts (Startup folders, Start menu, desktop) and Task Scheduler tasks that point to a
    program that no longer exists,
  - large files in Downloads (100 MB and more) and in the user profile (1 GB and more).
- **Safe by design:**
  - leftover folders and files go to the **Recycle Bin**,
  - every registry key or value is **exported to a `.reg` backup** before deletion (double-click the file to
    restore), every Task Scheduler task to an `.xml` backup, and a **System Restore point** is created first,
  - junctions and symlinks are removed as links and never followed,
  - files in use are skipped and every file that could not be deleted is written to the log,
  - the window can only pick items the scan found; it never sends a path to delete,
  - save games, large files and leftover folders that still contain `.exe` files (possibly portable programs)
    are never preselected.
- **Works on other people's PCs too:**
  - Windows installed on a drive other than `C:` is supported,
  - moved user folders (Documents in OneDrive, Downloads on another drive) are found through the Windows settings,
  - on a standard account, where UAC asks for a **different administrator's** password, PC Cleaner still cleans the
    profile, registry and Recycle Bin of the person who started it, not the administrator's,
  - the AI setup picks the ARM64 engine on ARM PCs and adds AMD (ROCm) libraries when an AMD GPU is present.
- **AI on demand.** The model never decides what gets deleted and never runs commands. It loads only when you
  ask for it: to describe what each leftover was, to describe the largest folders on a drive and suggest what to do,
  or to chat about the plan, folders and programs. AI calls run in the background, so the window stays responsive.
- **Updates.** PC Cleaner checks GitHub Releases at start. When a new version is out, one click downloads the signed
  installer, installs it and starts the new version.

## Installation

### Installer (recommended)

1. Download `PCCleaner-Setup-<version>.exe` from [Releases](https://github.com/Banzamel/PCCleaner/releases).
2. Run it. It installs for your account into `C:\Banzamel\PCCleaner` on the system drive (no admin rights needed for
   the installation; an update of the PowerShell version stays in its old folder) and lets you choose the language.
   It adds *PC Cleaner* and *PC Cleaner (preview)* to the Start menu.
3. Start **PC Cleaner**. Windows asks for administrator rights (UAC) - the program needs them to clean system caches.
4. Optional: *Settings → Install the AI assistant* downloads the Ollama engine (about 1.4 GB) and the model
   (`qwen3:4b-instruct` 2.5 GB or `qwen3:8b` 5.2 GB) **into the program folder** (`ai\`).

Everything stays inside the install folder: the program, its settings, logs, backups, the window's WebView2 data and
the AI engine. **PC Cleaner writes nothing to `%APPDATA%`, `%LOCALAPPDATA%` or your profile** (only `%TEMP%` during
setup and updates, and the shortcuts). No separate Ollama installation, no background service, no environment variables. A tray icon and an autostart task appear only when you turn them on in *Settings* (the X button
hiding to the tray, automatic cleaning). The AI engine runs on a private port (`11435`) only while PC Cleaner is open,
so it does not conflict with an Ollama you may have installed yourself. **Uninstalling removes the program and the automatic
cleaning task; tick *Delete the application data* to also remove `data\` (settings, logs, the `.reg`/`.xml` backups)
and the AI engine with its models.** Updating keeps all of them.

**Updating from the PowerShell version (1.8.0 and older):** the new installer installs into the old program folder,
moves the settings, logs, history and backups from `%LOCALAPPDATA%\PCCleaner` into `data\` (deleting the old folder
after checking the copy), keeps the AI engine and models and removes the old version silently. If the old version
cannot be removed, the installer stops and changes nothing. The update button of the old version installs it the same way.

The installer is not code-signed, so Windows SmartScreen may show a warning on the first run
(*More info → Run anyway*).

### Requirements

- Windows 10 or 11 (64-bit) with Microsoft Edge WebView2 (built in; the installer adds it when missing).
- Administrator rights. PC Cleaner asks for them through UAC at start.
- Optional, for the AI assistant: about 4–7 GB of disk space in the install folder. An NVIDIA GPU with 6 GB of
  VRAM or more is recommended, though it also runs (slower) on the CPU.

## Usage

Start **PC Cleaner** from the Start menu or the desktop. The Start menu also has *PC Cleaner (preview)*, which never
deletes anything.

### The window

The pages are in the navigation bar on the left.

1. **Overview**: tick the drives to scan (each row shows how full the drive is) and click **Scan** (or press **F5**).
   Afterwards the page shows how much space can be freed, three tiles (*Safe to delete*, *To review*, *Large files and
   folders*) that open the matching part of the plan, a chart of what takes it up and how many programs start with
   Windows, with **View the cleanup plan** and **Clean now...**. The *i* next to each page title explains the page.
2. **Cleanup plan**: groups start collapsed with their total; click a header to expand one. Each item has a type icon.
   The box on the left ticks or unticks an item, a click on its name shows the full description,
   and a click on a column header (name, size, confidence) sorts the list. Large folders and files (over 1 GB) are in
   their own tab, **Large files and folders**, and are never ticked in advance. Search and the
   *All / Selected / Opt-in* filter narrow the list, and **Select recommended** brings back the default plan.
   - **AI: what were the leftovers?** fills in what each leftover was (marked *AI (may be wrong)*),
   - **Clean selected...** shows a summary and asks for confirmation. A progress screen then shows every step, and
     the result: space freed, free space before and after as bars, backups, the restore point and the last five cleanups.
3. **What takes up space**: pick a drive, click **Analyse**, then **AI: describe and advise**. Double-click an
   item to show it in File Explorer.
4. **Startup apps**: the switch on each row turns a program on or off at once, like in Task Manager. Protected items
   are read-only. The bin next to the switch removes an entry permanently, with a `.reg` / `.xml` backup (shortcuts go
   to the Recycle Bin). **AI: what is it?** describes the programs. The change takes effect at the next sign-in.
   Double-click an item to show its file.
5. **Drivers**: **Check** looks for newer drivers on Windows Update (usually 1-3 minutes). Each row shows the
   device, the current and new version, and the download size; the box ticks or unticks it.
   **Install selected...** creates a System Restore point and installs the ticked drivers one by one. Devices with a
   problem in Device Manager are listed below; double-click opens Device Manager.
6. **AI assistant**: ask about the plan, folders, programs or games (general questions work before a scan too).
7. **Log**: everything the program does, also written to the log file.
8. **Settings**: language (Polish, English, Spanish), theme (light, dark or as in Windows), the AI assistant on or
   off, the AI model and **Install the AI assistant**, the update check at start, **the X button** (closes the program
   or hides the window to the notification area; *Exit* in the tray icon menu closes it), **automatic cleaning**
   (never, at every sign-in, once a day or once a week; applied as soon as you save), *always start in preview mode*,
   the logs (open the folder, clear them, delete ones older than 30 days at start), **What to scan** (each category,
   leftovers and large files on or off) and **Excluded folders**, which are never measured, listed or deleted.
   **Save and restart** applies them at once. They are kept in `data\settings.json` in the program folder;
   command-line parameters win over them.

**Ctrl+1** to **Ctrl+8** switch pages. The window follows the Windows light or dark app mode (unless *Settings*
says otherwise) and remembers its size, position and last page in `data\window.json`.

### Parameters

```
PCCleaner.exe                      the window
PCCleaner.exe --preview            preview: scan and plan only, nothing is deleted (also -Preview, -Podglad)
PCCleaner.exe --no-ai              without the AI assistant (also -NoAI, -BezLLM)
PCCleaner.exe --model qwen3:8b     another model
PCCleaner.exe --drives C,P         pre-select these drives (also -Drives, -Dyski)
PCCleaner.exe --language en        interface language: en, pl or es (also -Language, -Jezyk)
PCCleaner.exe --no-update-check    no update check at start (also -NoUpdateCheck, -BezAktualizacji)
PCCleaner.exe --auto-clean         automatic cleaning, as started by the Settings task at sign-in
```

### Drives

PC Cleaner lists local fixed drives with a letter. USB drives, memory cards, network drives and partitions without
a letter (EFI, Recovery) are skipped, as are BitLocker-locked volumes. All drives are ticked by default; the system
drive is always scanned (Windows caches, user profile, Program Files). The other ticked drives are used for the
Recycle Bin, Steam and Epic libraries, and the *what takes up space* view.

### What gets preselected

| Item | Rule |
|---|---|
| Cleanup categories | any non-empty category, however small. The Recycle Bin (irreversible), DNS, JetBrains indexes, `Windows.old`, DISM, older driver versions and both Docker categories are never preselected |
| Leftovers, *high* confidence | Steam/Epic leftovers, dead uninstall entries, dead startup entries, Start menu shortcuts and tasks, folders unchanged for 180 days or more |
| Leftovers, *medium* confidence | folders unchanged for 30–180 days, orphaned registry keys, dead desktop shortcuts |
| Leftovers, *low* confidence | save games, large files: **never preselected** |

A folder is **not** treated as a leftover when its name or path matches an installed program, a Microsoft Store app,
a service, a running process, a Start menu or desktop shortcut, or an installed Steam game. It is also skipped when it
was modified in the last 30 days or is on the built-in list of system and driver folders
(`SKIPPED_NAMES` in `src-tauri/core/src/scan/leftovers/names.rs`).

## Files and logs

| Path | Contents |
|---|---|
| install folder (`C:\Banzamel\PCCleaner` by default) | `PCCleaner.exe`, `uninstall.exe`, `config.json` (the model chosen when the AI was installed) |
| `<install folder>\ai\ollama\` | the Ollama engine |
| `<install folder>\ai\models\` | AI models |
| `<install folder>\data\webview\` | the window's WebView2 data (cache) |
| `<install folder>\data\logs\` | one log per run: scan report, chosen plan, results, everything moved to the Recycle Bin, files that could not be deleted |
| `<install folder>\data\window.json` | window size, position, last page and theme |
| `<install folder>\data\settings.json` | the *Settings* page, including the date of the last automatic cleaning |
| `<install folder>\data\history.json` | the last 20 cleanups |
| Task Scheduler: `PC Cleaner - automatic cleaning` | only when automatic cleaning is on in *Settings* |
| `<install folder>\data\logs\kopie_rejestru\` | `.reg` backups of deleted registry keys and values, `.xml` backups of deleted Task Scheduler tasks |

## About the AI part

The assistant is deliberately limited. During development a 4B model planned the cleanup by itself. The result
was almost identical to a simple size rule, but it took 50–100 seconds and some explanations were wrong. When asked
about game studios, the model also invented titles. That is why:

- the plan is built by rules and the scanner provides the facts (sizes, dates, registry subkeys),
- the model only describes and answers, and is told not to add facts that are not in the data,
- its descriptions are labelled *AI (may be wrong)*,
- its advice in the disk analysis comes from a fixed list of safe options, and for system and program folders the
  program sets the advice itself.

| Model | Notes (tested on a GTX 1060 6 GB) |
|---|---|
| `qwen3:4b-instruct` | **recommended**, runs fully on the GPU, describing the leftovers takes about 30–60 s |
| `qwen3:8b` | does not fit into 6 GB VRAM, about 3× slower; worth it with 8 GB or more |

## Troubleshooting

| Problem | Fix |
|---|---|
| *The AI assistant is not installed* | *Settings → Install the AI assistant* |
| *Could not start Ollama* | run `ai\ollama\ollama.exe serve` from the install folder and check the output |
| *Model … is missing* | *Settings → Install the AI assistant* with that model |
| AI answers take minutes | the model probably does not fit into the VRAM (check *Dedicated GPU memory* in Task Manager); close games or other GPU-heavy programs |
| Browser cache skipped | close the browser (also from the system tray) and run again |
| Taskbar did not come back | `Ctrl+Shift+Esc` → File → Run new task → `explorer.exe` |
| A registry key was needed after all | double-click its `.reg` file in `logs\kopie_rejestru\`, or use the *PC Cleaner* System Restore point |
| A deleted scheduled task was needed after all | Task Scheduler → *Import Task...* → its `.xml` file in `logs\kopie_rejestru\` |
| A program no longer starts with Windows | *Startup apps* page → turn its switch on (or Task Manager → *Startup apps*) |
| Restore point *skipped by Windows* | Windows creates at most one restore point per 24 hours by default; the `.reg` backups are still made |
| App cache *skipped: … is running* | close the app (also from the system tray) and run again |

## Code structure

PC Cleaner is a [Tauri 2](https://tauri.app) program: the system work is done in Rust, the window is React 19 with
TypeScript (Vite).

| Folder | Contents |
|---|---|
| `src-tauri/core/` | crate `pccleaner-core`, no UI: scanning, cleanup categories (one trait, one file per category), leftovers, cleaning, startup apps, drivers, disk analysis, the Ollama client, storage |
| `src-tauri/src/` | the Tauri app: typed commands, events, tray, window state, elevation |
| `src/app/` | the window shell: navigation, boot, the store and the services |
| `src/features/<feature>/` | one folder per page: `components/`, `hooks/`, services, `index.ts` |
| `src/core/` | pure TypeScript logic: plan rules, summary, AI prompts and report |
| `src/platform/` | the only code that talks to Tauri |
| `src/lang/` | `pl.json`, `es.json`: the English text mapped to the translation |

The rules every file follows (size limits, one responsibility per file, feature folders, services as classes, strict
TypeScript, Rust module layout, tests mirroring the source tree) are in [`CLAUDE.md`](CLAUDE.md) under *Code structure*.

## Building from source

Requirements: Node.js 22+, Rust (stable, MSVC toolchain) on Windows. Then:

```powershell
npm ci
npm test                                   # TypeScript tests (Vitest)
cd src-tauri; cargo test -p pccleaner-core; cd ..
npx tauri build                            # installer in src-tauri\target\release\bundle\nsis\
```

Without Rust on the machine, `npm run dist` builds the installer in Docker (`docker/build.sh`, image `tauri-build`
with cargo-xwin and NSIS) and copies it to `release\`.

The installer is built and published by GitHub Actions on every `v*` tag (`.github/workflows/release.yml`). To
release, set the version in `package.json`, rename *Unreleased* in `CHANGELOG.md` to that version, then tag:

```powershell
git tag -a v2.0.0 -m "PC Cleaner 2.0.0"
git push origin main --tags
```

The workflow fails if the tag does not match `package.json`. It publishes `PCCleaner-Setup-<version>.exe`
and `latest.json` (for the updater, with the signature inside).

## License

[MIT](LICENSE)
