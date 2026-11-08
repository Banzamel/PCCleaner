; PC Cleaner - hooks for the Tauri NSIS installer.
;
; 1. Moving from the PowerShell version (Inno Setup, AppId {6F3C2B1E-8A4D-4E7B-9C51-2D7E0F4A9B63}):
;    its uninstaller deletes %LOCALAPPDATA%\PCCleaner and {app}\ai, so the settings, logs and
;    registry backups are set aside and the AI engine with its models is moved into the new folder
;    before the old version is removed silently.
; 2. First install: the program starts in the language chosen in the installer.
; 3. Uninstall: the AI engine, the automatic cleaning task and %LOCALAPPDATA%\PCCleaner go too - but
;    never during an update ($UpdateMode) or when a newer installer runs this uninstaller in place.

!define PCC_OLD_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\{6F3C2B1E-8A4D-4E7B-9C51-2D7E0F4A9B63}_is1"
!define PCC_TASK "PC Cleaner - automatic cleaning"

!macro NSIS_HOOK_PREINSTALL
  SetShellVarContext current
  ReadRegStr $R8 HKCU "${PCC_OLD_KEY}" "UninstallString"
  ReadRegStr $R9 HKCU "${PCC_OLD_KEY}" "InstallLocation"
  ${If} $R8 == ""
    ReadRegStr $R8 HKLM "${PCC_OLD_KEY}" "UninstallString"
    ReadRegStr $R9 HKLM "${PCC_OLD_KEY}" "InstallLocation"
  ${EndIf}
  ${If} $R8 != ""
    DetailPrint "PC Cleaner (PowerShell) -> PC Cleaner"
    ; The install folder from the uninstaller path when InstallLocation is missing; always with a trailing \
    ${If} $R9 == ""
      StrCpy $R6 $R8 "" 1
      StrCpy $R6 $R6 -1
      ${GetParent} $R6 $R9
    ${EndIf}
    StrCpy $R7 $R9 1 -1
    ${If} $R7 != "\"
      StrCpy $R9 "$R9\"
    ${EndIf}
    ; Settings, window placement, history, logs and .reg/.xml backups
    ${If} ${FileExists} "$LOCALAPPDATA\PCCleaner\*.*"
    ${AndIfNot} ${FileExists} "$LOCALAPPDATA\PCCleaner.migrate\*.*"
      Rename "$LOCALAPPDATA\PCCleaner" "$LOCALAPPDATA\PCCleaner.migrate"
    ${EndIf}
    ; The AI engine and the models (several GB): moved when on the same drive, copied otherwise
    ${If} ${FileExists} "$R9ai\ollama\ollama.exe"
    ${AndIfNot} ${FileExists} "$INSTDIR\ai\ollama\ollama.exe"
      CreateDirectory "$INSTDIR"
      ClearErrors
      Rename "$R9ai" "$INSTDIR\ai"
      ${If} ${Errors}
        CreateDirectory "$INSTDIR\ai"
        CopyFiles /SILENT "$R9ai\*.*" "$INSTDIR\ai"
      ${EndIf}
      CopyFiles /SILENT "$R9config.json" "$INSTDIR"
    ${EndIf}
    ExecWait '$R8 /VERYSILENT /SUPPRESSMSGBOXES /NORESTART'
    ; The Inno uninstaller finishes in a copy of itself; wait until it removed unins000.exe (max 60 s)
    StrCpy $R7 0
    pcc_wait_uninstall:
      IfFileExists "$R9unins000.exe" 0 pcc_uninstalled
      IntCmp $R7 120 pcc_uninstalled
      Sleep 500
      IntOp $R7 $R7 + 1
      Goto pcc_wait_uninstall
    pcc_uninstalled:
    ${If} ${FileExists} "$LOCALAPPDATA\PCCleaner.migrate\*.*"
      RMDir /r "$LOCALAPPDATA\PCCleaner"
      Rename "$LOCALAPPDATA\PCCleaner.migrate" "$LOCALAPPDATA\PCCleaner"
    ${EndIf}
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTINSTALL
  SetShellVarContext current
  ${IfNot} ${FileExists} "$LOCALAPPDATA\PCCleaner\settings.json"
    StrCpy $R7 "en"
    ${If} $LANGUAGE == 1045
      StrCpy $R7 "pl"
    ${ElseIf} $LANGUAGE == 1034
    ${OrIf} $LANGUAGE == 3082
      StrCpy $R7 "es"
    ${EndIf}
    CreateDirectory "$LOCALAPPDATA\PCCleaner"
    FileOpen $R6 "$LOCALAPPDATA\PCCleaner\settings.json" w
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
    RMDir /r "$INSTDIR\ai"
    Delete "$INSTDIR\config.json"
    Delete "$SMPROGRAMS\${PRODUCTNAME} (preview).lnk"
    Delete "$SMPROGRAMS\${PRODUCTNAME} (podgląd).lnk"
    Delete "$SMPROGRAMS\${PRODUCTNAME} (vista previa).lnk"
    SetShellVarContext current
    RMDir /r "$LOCALAPPDATA\PCCleaner"
  ${EndIf}
!macroend
