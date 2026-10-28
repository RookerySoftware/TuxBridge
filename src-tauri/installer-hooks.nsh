; Hooks into Tauri's NSIS installer (tauri.conf.json, bundle.windows.nsis.installerHooks).

; TuxBridge used to be called EasyWSL, then Pengi: separate apps as far as Windows is concerned.
; Remove them first, silently, so they don't run side by side. Their uninstallers only delete
; their own program files: the data folder (%LOCALAPPDATA%\EasyWSL or \Pengi: settings, logs,
; Docker's certificates, and the systems' disks) stays, and TuxBridge keeps using it.
!macro TUXBRIDGE_REMOVE_EARLIER NAME
  ReadRegStr $R9 HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${NAME}" "InstallLocation"
  ${If} $R9 == ""
    StrCpy $R9 "$LOCALAPPDATA\${NAME}"
  ${EndIf}
  ${If} ${FileExists} "$R9\uninstall.exe"
    DetailPrint "Removing ${NAME}, TuxBridge's earlier name (your systems and settings stay)"
    ExecWait '"$R9\uninstall.exe" /S _?=$R9'
    Delete "$R9\uninstall.exe"
  ${EndIf}
!macroend

!macro NSIS_HOOK_PREINSTALL
  Push $R9
  !insertmacro TUXBRIDGE_REMOVE_EARLIER "EasyWSL"
  !insertmacro TUXBRIDGE_REMOVE_EARLIER "Pengi"
  Pop $R9
!macroend
