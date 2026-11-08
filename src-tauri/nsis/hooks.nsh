; PC Cleaner - hooks for the Tauri NSIS installer.
;
; Nothing of PC Cleaner lives in the user profile: the program, its data (<install>\data) and the AI
; engine (<install>\ai) all stay in one folder. Only %TEMP% (during setup) and the shortcuts are outside.
;
; 1. Install folder: the folder of the PowerShell version (Inno Setup) when it is installed, otherwise
;    <system drive>\Banzamel\PCCleaner. Updates reuse the current folder (Tauri remembers it).
; 2. Moving from the PowerShell version: its uninstaller deletes {app}\ai, {app}\config.json and
;    %LOCALAPPDATA%\PCCleaner, so they are parked first, the old version is removed silently, and only
;    when it is really gone are they moved into the new folder (pcc-migrate.nsh).
; 3. Data of older versions in %LOCALAPPDATA%\PCCleaner is copied to <install>\data, checked, then deleted.
; 4. Uninstall: the automatic cleaning task always goes; data and the AI engine only when the user ticks
;    "Delete the application data". Never during an update or when a newer installer runs it in place.

!include "${__FILEDIR__}\pcc-migrate.nsh"

; Runs before the pages: a fresh install (Tauri's default in %LOCALAPPDATA%) gets our folder instead
!define MUI_CUSTOMFUNCTION_GUIINIT PccChooseDir
Function PccChooseDir
  Call PccFixInstDir
FunctionEnd

!macro NSIS_HOOK_PREINSTALL
  SetShellVarContext current
  ; A silent install skips the pages, so the folder is checked here too. The template has already
  ; created Tauri's default folder (SetOutPath); remove it again when it stays empty.
  StrCpy $PccOrigDir $INSTDIR
  Call PccFixInstDir
  ${If} $PccOrigDir != $INSTDIR
    RMDir $PccOrigDir
  ${EndIf}
  SetOutPath $INSTDIR
  Call PccMigrateInno
!macroend

!macro NSIS_HOOK_POSTINSTALL
  SetShellVarContext current
  Call PccMigrateData
  ${IfNot} ${FileExists} "$INSTDIR\data\settings.json"
    StrCpy $R7 "en"
    ${If} $LANGUAGE == 1045
      StrCpy $R7 "pl"
    ${ElseIf} $LANGUAGE == 1034
    ${OrIf} $LANGUAGE == 3082
      StrCpy $R7 "es"
    ${EndIf}
    CreateDirectory "$INSTDIR\data"
    FileOpen $R6 "$INSTDIR\data\settings.json" w
    FileWrite $R6 '{"Language": "$R7"}'
    FileClose $R6
  ${EndIf}
  ; A second Start menu entry that never deletes anything
  StrCpy $R7 "preview"
  ${If} $LANGUAGE == 1045
    StrCpy $R7 "podgląd"
  ${ElseIf} $LANGUAGE == 1034
  ${OrIf} $LANGUAGE == 3082
    StrCpy $R7 "vista previa"
  ${EndIf}
  CreateShortCut "$SMPROGRAMS\${PRODUCTNAME} ($R7).lnk" "$INSTDIR\${MAINBINARYNAME}.exe" "--preview" "$INSTDIR\${MAINBINARYNAME}.exe" 0
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ${If} $UpdateMode <> 1
  ${AndIf} "$EXEDIR" != "$INSTDIR"
    !insertmacro CheckIfAppIsRunning "$INSTDIR\${MAINBINARYNAME}.exe" "${PRODUCTNAME}"
    nsExec::Exec '"$SYSDIR\schtasks.exe" /Delete /TN "${PCC_TASK}" /F'
    Pop $R7
    Delete "$SMPROGRAMS\${PRODUCTNAME} (preview).lnk"
    Delete "$SMPROGRAMS\${PRODUCTNAME} (podgląd).lnk"
    Delete "$SMPROGRAMS\${PRODUCTNAME} (vista previa).lnk"
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ; Settings, logs, backups and the AI engine with its models (several GB) only when the user asked
  ${If} $UpdateMode <> 1
  ${AndIf} "$EXEDIR" != "$INSTDIR"
  ${AndIf} $DeleteAppDataCheckboxState = 1
    RMDir /r "$INSTDIR\data"
    RMDir /r "$INSTDIR\ai"
    Delete "$INSTDIR\config.json"
  ${EndIf}
  RMDir "$INSTDIR"
!macroend
