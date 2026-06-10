; NSIS installer hooks for JX3-Tools
;
; ⚠️ 刻意不在安装期触碰任何内核驱动！
; 官方 install-interception.exe /install 会同时装键盘+鼠标两个 class 过滤驱动
; （没有只装键盘的参数），鼠标过滤器曾导致用户鼠标瘫痪（2026-06 事故）。
; 本工具已**不再使用官方安装器**：键盘驱动改为应用内"按键"页面由用户知情后
; 手动安装，且完全自己实现「只装键盘」——拷 keyboard.sys + 注册键盘驱动服务 +
; 只往键盘 class 的 UpperFilters 加项，全程绝不写任何鼠标相关项
; （见 src-tauri/src/services/hotkey/driver.rs）。
; 不要在这里恢复任何形式的自动驱动安装。

!macro NSIS_HOOK_PREINSTALL
!macroend

!macro NSIS_HOOK_POSTINSTALL
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; 不自动卸载驱动：卸载需重启生效，且其他软件可能也在使用键盘过滤器。
  ; 如需卸载：在应用内"按键"页面操作（删服务/过滤器/keyboard.sys）。
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
!macroend
