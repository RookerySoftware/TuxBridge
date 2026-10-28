; Hooks into Tauri's NSIS installer (tauri.conf.json, bundle.windows.nsis.installerHooks).

; Pengi used to be called EasyWSL, a separate app as far as Windows is concerned. Remove that
; app first, silently, so the two don't run side by side. Its uninstaller only deletes its own
; program files: the data folder (%LOCALAPPDATA%\EasyWSL: settings, logs, Docker's certificates,
; and the systems' disks) stays, and Pengi keeps using it.
!macro NSIS_HOOK_PREINSTALL
  Push $R9
  ReadRegStr $R9 HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\EasyWSL" "InstallLocation"
  ${If} $R9 == ""
    StrCpy $R9 "$LOCALAPPDATA\EasyWSL"
  ${EndIf}
  ${If} ${FileExists} "$R9\uninstall.exe"
    DetailPrint "Removing EasyWSL, Pengi's earlier name (your systems and settings stay)"
    ExecWait '"$R9\uninstall.exe" /S _?=$R9'
    Delete "$R9\uninstall.exe"
  ${EndIf}
  Pop $R9
!macroend
