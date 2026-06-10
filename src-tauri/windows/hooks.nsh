; NSIS installer hooks for JX3-Tools
;
; ⚠️ 刻意不在安装期触碰 Interception 内核驱动！
; install-interception.exe /install 会同时安装键盘+鼠标两个 class 过滤驱动
; （没有只装键盘的参数），鼠标过滤器曾导致用户鼠标瘫痪（2026-06 事故：
; UpperFilters 引用的过滤器加载失败时整个鼠标设备栈起不来，重启也无效）。
; 驱动安装已改为应用内"按键"页面由用户知情后手动触发：只保留键盘过滤器，
; 安装完成后立即从注册表移除鼠标过滤器（见 src/services/hotkey/driver.rs）。
; 不要在这里恢复任何形式的自动驱动安装。

!macro NSIS_HOOK_PREINSTALL
!macroend

!macro NSIS_HOOK_POSTINSTALL
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; 不自动卸载驱动：卸载需重启生效，且其他软件可能也在使用 Interception。
  ; 如需卸载：应用内"按键"页面操作，或以管理员身份运行
  ; install-interception.exe /uninstall（随后重启）。
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
!macroend
