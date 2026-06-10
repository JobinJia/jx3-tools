; NSIS installer hooks for JX3-Tools
; Installs the Interception kernel driver bundled under resources/interception.
; The driver lets key simulation reach JX3 (its anti-cheat filters user-mode
; synthesized input). A reboot is required before the driver becomes active.

!macro NSIS_HOOK_PREINSTALL
!macroend

!macro NSIS_HOOK_POSTINSTALL
  DetailPrint "正在安装 Interception 按键驱动..."

  ; interception.dll must sit next to the exe so it loads at runtime;
  ; Tauri puts bundled resources under $INSTDIR\resources\, not beside the exe.
  CopyFiles /SILENT "$INSTDIR\resources\interception\interception.dll" "$INSTDIR\interception.dll"

  ; Install the kernel driver (needs admin; the app manifest already elevates the installer)
  nsExec::ExecToLog '"$INSTDIR\resources\interception\install-interception.exe" /install'
  Pop $0
  DetailPrint "Interception 驱动安装返回码: $0"

  MessageBox MB_OK|MB_ICONINFORMATION "按键驱动已安装。$\r$\n请重启电脑后，按键功能才能在游戏内生效。"
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; 默认不卸载 Interception 驱动：其他软件可能也在使用它。
  ; 如确需卸载，请手动以管理员身份运行：install-interception.exe /uninstall（需重启）。
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
!macroend
