; PC Cleaner - install folder choice and the move from the PowerShell version (included by hooks.nsh).
; Note for editors: makensis joins a line that ends in a backslash with the next one, also in comments.

!define PCC_OLD_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\{6F3C2B1E-8A4D-4E7B-9C51-2D7E0F4A9B63}_is1"
!define PCC_TASK "PC Cleaner - automatic cleaning"
!define PCC_OLD_DATA "$LOCALAPPDATA\PCCleaner"

; Tauri's default folder as the template set it, before PccFixInstDir redirects the install
Var PccOrigDir

; Out: $R4 = UninstallString of the PowerShell version ("" when it is not installed),
;      $R5 = its folder without the trailing backslash
Function PccOldDir
  ReadRegStr $R4 HKCU "${PCC_OLD_KEY}" "UninstallString"
  ReadRegStr $R5 HKCU "${PCC_OLD_KEY}" "InstallLocation"
  ${If} $R4 == ""
    ReadRegStr $R4 HKLM "${PCC_OLD_KEY}" "UninstallString"
    ReadRegStr $R5 HKLM "${PCC_OLD_KEY}" "InstallLocation"
  ${EndIf}
  ${If} $R4 != ""
  ${AndIf} $R5 == ""
    ; "C:\...\unins000.exe" without the quotes, then its folder
    StrCpy $R5 $R4 "" 1
    StrCpy $R5 $R5 -1
    ${GetParent} $R5 $R5
  ${EndIf}
  StrCpy $R3 $R5 1 -1
  ${If} $R3 == "\"
    StrCpy $R5 $R5 -1
  ${EndIf}
FunctionEnd

; A fresh per-user install (Tauri's default folder in the profile) goes into the folder of the
; PowerShell version, or <system drive>\Banzamel\PCCleaner. A per-machine old folder under
; Program Files is not writable without administrator rights, so it gets the default too.
Function PccFixInstDir
  ; Tauri's default for a per-user install ("$LOCALAPPDATA\<product name>"; PRODUCTNAME is defined later)
  ${If} $INSTDIR != "$LOCALAPPDATA\PC Cleaner"
    Return
  ${EndIf}
  Call PccOldDir
  StrLen $R2 $PROGRAMFILES64
  StrCpy $R1 $R5 $R2
  StrLen $R2 $PROGRAMFILES
  StrCpy $R2 $R5 $R2
  ${If} $R5 != ""
  ${AndIf} ${FileExists} "$R5\*.*"
  ${AndIf} $R1 != $PROGRAMFILES64
  ${AndIf} $R2 != $PROGRAMFILES
    StrCpy $INSTDIR $R5
  ${Else}
    StrCpy $R1 $WINDIR 2
    StrCpy $INSTDIR "$R1\Banzamel\PCCleaner"
  ${EndIf}
FunctionEnd

; Moves a folder: Push source, Push target. Renamed on the same drive, copied otherwise; the source
; is removed only after a copy without errors.
Function PccMove
  Exch $R2
  Exch
  Exch $R1
  ClearErrors
  Rename $R1 $R2
  ${If} ${Errors}
    ClearErrors
    CreateDirectory $R2
    CopyFiles /SILENT "$R1\*.*" $R2
    ${IfNot} ${Errors}
      RMDir /r $R1
    ${EndIf}
  ${EndIf}
  Pop $R1
  Pop $R2
FunctionEnd

; Copies the data of older versions (%LOCALAPPDATA%\PCCleaner) to the folder on the stack unless it
; already holds settings. Out: $R7 = 1 when the copy was made and checked (the source may go).
Function PccCopyData
  Exch $R8
  StrCpy $R7 0
  ${If} ${FileExists} "${PCC_OLD_DATA}\*.*"
  ${AndIfNot} ${FileExists} "$R8\settings.json"
    CreateDirectory $R8
    ClearErrors
    CopyFiles /SILENT "${PCC_OLD_DATA}\*.*" $R8
    StrCpy $R7 1
    ${If} ${Errors}
      StrCpy $R7 0
    ${EndIf}
    ${If} ${FileExists} "${PCC_OLD_DATA}\settings.json"
    ${AndIfNot} ${FileExists} "$R8\settings.json"
      StrCpy $R7 0
    ${EndIf}
    ${If} ${FileExists} "${PCC_OLD_DATA}\history.json"
    ${AndIfNot} ${FileExists} "$R8\history.json"
      StrCpy $R7 0
    ${EndIf}
    ${If} ${FileExists} "${PCC_OLD_DATA}\logs\*.*"
    ${AndIfNot} ${FileExists} "$R8\logs\*.*"
      StrCpy $R7 0
    ${EndIf}
  ${EndIf}
  Pop $R8
FunctionEnd

; Data left by an older version without the Inno uninstaller (portable, or a failed uninstall earlier)
Function PccMigrateData
  Push "$INSTDIR\data"
  Call PccCopyData
  ${If} $R7 == 1
    RMDir /r "${PCC_OLD_DATA}"
  ${EndIf}
FunctionEnd

; The PowerShell version: park, uninstall, check it is really gone, then move into the new folder.
Function PccMigrateInno
  Call PccOldDir
  ${If} $R4 == ""
    Return
  ${EndIf}
  DetailPrint "PC Cleaner (PowerShell) -> PC Cleaner: $R5"
  ; The old uninstaller empties its own folder, so parked items wait next to it when we reuse it
  ${If} $R5 == $INSTDIR
    StrCpy $R6 "$R5.migrate"
  ${Else}
    StrCpy $R6 $INSTDIR
  ${EndIf}
  CreateDirectory $R6
  ${If} ${FileExists} "$R5\ai\*.*"
  ${AndIfNot} ${FileExists} "$R6\ai\*.*"
    Push "$R5\ai"
    Push "$R6\ai"
    Call PccMove
  ${EndIf}
  ${If} ${FileExists} "$R5\config.json"
    CopyFiles /SILENT "$R5\config.json" $R6
  ${EndIf}
  Push "$R6\data"
  Call PccCopyData
  ExecWait '$R4 /VERYSILENT /SUPPRESSMSGBOXES /NORESTART'
  ; The Inno uninstaller finishes in a copy of itself; wait until unins000.exe is gone (max 60 s)
  StrCpy $R3 0
  pcc_wait_uninstall:
    IfFileExists "$R5\unins000.exe" 0 pcc_uninstalled
    IntCmp $R3 120 pcc_uninstalled
    Sleep 500
    IntOp $R3 $R3 + 1
    Goto pcc_wait_uninstall
  pcc_uninstalled:
  ; Removed only when its uninstaller, its start script and its uninstall entry are all gone
  ReadRegStr $R3 HKCU "${PCC_OLD_KEY}" "UninstallString"
  ${If} $R3 == ""
    ReadRegStr $R3 HKLM "${PCC_OLD_KEY}" "UninstallString"
  ${EndIf}
  ${If} ${FileExists} "$R5\unins000.exe"
  ${OrIf} ${FileExists} "$R5\PCCleaner.ps1"
  ${OrIf} $R3 != ""
    ; Put the AI engine back and stop: the old version stays complete and usable
    ${If} ${FileExists} "$R6\ai\*.*"
    ${AndIfNot} ${FileExists} "$R5\ai\*.*"
      Push "$R6\ai"
      Push "$R5\ai"
      Call PccMove
    ${EndIf}
    ${If} $R6 != $INSTDIR
      RMDir /r $R6
    ${EndIf}
    MessageBox MB_ICONSTOP|MB_OK "The previous PC Cleaner (1.8 or older) could not be removed. Uninstall it in Settings > Apps, then run this installer again. Nothing was changed." /SD IDOK
    Abort
  ${EndIf}
  ${If} $R6 != $INSTDIR
    CreateDirectory $INSTDIR
    ${If} ${FileExists} "$R6\ai\*.*"
      Push "$R6\ai"
      Push "$INSTDIR\ai"
      Call PccMove
    ${EndIf}
    ${If} ${FileExists} "$R6\config.json"
      CopyFiles /SILENT "$R6\config.json" $INSTDIR
    ${EndIf}
    ${If} ${FileExists} "$R6\data\*.*"
    ${AndIfNot} ${FileExists} "$INSTDIR\data\*.*"
      Push "$R6\data"
      Push "$INSTDIR\data"
      Call PccMove
    ${EndIf}
    RMDir /r $R6
  ${EndIf}
  ; The copy was checked before the uninstaller ran; the old uninstaller normally deleted it already
  ${If} $R7 == 1
    RMDir /r "${PCC_OLD_DATA}"
  ${EndIf}
FunctionEnd
