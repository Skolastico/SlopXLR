Unicode true
!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "FileFunc.nsh"
!include "x64.nsh"
!ifndef VERSION
 !define VERSION "0.1.0-beta.1"
!endif
!ifndef PACKAGE_DIR
 !error "PACKAGE_DIR is required"
!endif
!ifndef OUTPUT
 !define OUTPUT "SlopXLR-setup.exe"
!endif
Name "SlopXLR ${VERSION}"
OutFile "${OUTPUT}"
InstallDir "$LOCALAPPDATA\Programs\GoXLR-Personalizado"
InstallDirRegKey HKCU "Software\Skolastico\GoXLR-Personalizado" "InstallDir"
RequestExecutionLevel user
SetCompressor /SOLID lzma
ShowInstDetails show
ShowUninstDetails show
!define MUI_ABORTWARNING
!define MUI_ICON "../../daemon/resources/goxlr-utility.ico"
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "../../LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "Portuguese"
LangString DriverMissing ${LANG_ENGLISH} "The GoXLR audio driver was not found. Install the official GoXLR driver, then run this installer again."
LangString DriverMissing ${LANG_PORTUGUESE} "O driver de áudio GoXLR não foi encontrado. Instala o driver oficial GoXLR e volta a executar este instalador."
LangString Running ${LANG_ENGLISH} "Quit GoXLR Utility and the official GoXLR app from their tray icons before installing."
LangString Running ${LANG_PORTUGUESE} "Encerra o GoXLR Utility e a app oficial GoXLR pelos ícones junto ao relógio antes de instalar."
LangString WebviewMissing ${LANG_ENGLISH} "WebView2 installation failed. Install Microsoft Edge WebView2 Runtime and try again."
LangString WebviewMissing ${LANG_PORTUGUESE} "A instalação do WebView2 falhou. Instala o Microsoft Edge WebView2 Runtime e volta a tentar."
LangString StartLabel ${LANG_ENGLISH} "Start with Windows"
LangString StartLabel ${LANG_PORTUGUESE} "Iniciar com o Windows"
LangString ShortcutLabel ${LANG_ENGLISH} "Desktop shortcut"
LangString ShortcutLabel ${LANG_PORTUGUESE} "Atalho no ambiente de trabalho"
Var Sandbox
Function .onInit
 SetShellVarContext current
 SetRegView 64
 ${GetParameters} $0
 ClearErrors
 ${GetOptions} $0 "/SANDBOX" $1
 ${IfNot} ${Errors}
  StrCpy $Sandbox 1
 ${EndIf}
 ${IfNot} ${RunningX64}
  MessageBox MB_ICONSTOP "Windows x64 is required."
  SetErrorLevel 1
  Abort
 ${EndIf}
 !insertmacro MUI_LANGDLL_DISPLAY
FunctionEnd
Function Preflight
 ${If} $Sandbox == 1
  Return
 ${EndIf}
 InitPluginsDir
 SetOutPath "$PLUGINSDIR"
 File "${PACKAGE_DIR}\Preflight.ps1"
 nsExec::ExecToStack '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$PLUGINSDIR\Preflight.ps1"'
 Pop $0
 Pop $1
 ${If} $0 == 2
  MessageBox MB_ICONSTOP "$(DriverMissing)"
  SetErrorLevel 2
  Abort
 ${ElseIf} $0 != 0
  MessageBox MB_ICONSTOP "$(Running)"
  SetErrorLevel 3
  Abort
 ${EndIf}
FunctionEnd
Section "SlopXLR (beta)" Main
 SectionIn RO
 Call Preflight
 SetOutPath "$INSTDIR"
 File /r "${PACKAGE_DIR}\*"
 FileOpen $0 "$INSTDIR\installed.marker" w
 FileWrite $0 "installed"
 FileClose $0
 WriteUninstaller "$INSTDIR\Uninstall.exe"
 ${If} $Sandbox != 1
  WriteRegStr HKCU "Software\Skolastico\GoXLR-Personalizado" "InstallDir" "$INSTDIR"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\GoXLR-Personalizado" "DisplayName" "SlopXLR"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\GoXLR-Personalizado" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\GoXLR-Personalizado" "Publisher" "Skolastico"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\GoXLR-Personalizado" "UninstallString" '$\"$INSTDIR\Uninstall.exe$\"'
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\GoXLR-Personalizado" "DisplayIcon" "$INSTDIR\goxlr-daemon.exe"
  WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\GoXLR-Personalizado" "NoModify" 1
  WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\GoXLR-Personalizado" "NoRepair" 1
  ; Migrate shortcuts created by the first beta; retain configuration paths.
  Delete "$SMPROGRAMS\GoXLR Personalizado\GoXLR Personalizado.lnk"
  RMDir "$SMPROGRAMS\GoXLR Personalizado"
  IfFileExists "$SMSTARTUP\GoXLR Personalizado.lnk" 0 +3
   CreateShortcut "$SMSTARTUP\SlopXLR.lnk" "$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" '-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File "$INSTDIR\Start.ps1" -Background' "$INSTDIR\goxlr-daemon.exe" 0 SW_SHOWMINIMIZED
   Delete "$SMSTARTUP\GoXLR Personalizado.lnk"
  IfFileExists "$DESKTOP\GoXLR Personalizado.lnk" 0 +3
   CreateShortcut "$DESKTOP\SlopXLR.lnk" "$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" '-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File "$INSTDIR\Start.ps1"' "$INSTDIR\goxlr-daemon.exe" 0 SW_SHOWMINIMIZED
   Delete "$DESKTOP\GoXLR Personalizado.lnk"
  CreateDirectory "$SMPROGRAMS\SlopXLR"
  CreateShortcut "$SMPROGRAMS\SlopXLR\SlopXLR.lnk" "$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" '-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File "$INSTDIR\Start.ps1"' "$INSTDIR\goxlr-daemon.exe" 0 SW_SHOWMINIMIZED
  ReadRegStr $0 HKLM "SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" "pv"
  ReadRegStr $1 HKCU "SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" "pv"
  ${If} $0 == ""
  ${AndIf} $1 == ""
   ExecWait '"$INSTDIR\WebView2Setup.exe" /silent /install' $2
   ${If} $2 != 0
    MessageBox MB_ICONSTOP "$(WebviewMissing)"
    SetErrorLevel 4
   ${EndIf}
  ${EndIf}
 ${EndIf}
SectionEnd
Section /o "$(StartLabel)" AutoStart
 ${If} $Sandbox != 1
  CreateShortcut "$SMSTARTUP\SlopXLR.lnk" "$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" '-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File "$INSTDIR\Start.ps1" -Background' "$INSTDIR\goxlr-daemon.exe" 0 SW_SHOWMINIMIZED
 ${EndIf}
SectionEnd
Section /o "$(ShortcutLabel)" DesktopShortcut
 ${If} $Sandbox != 1
  CreateShortcut "$DESKTOP\SlopXLR.lnk" "$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" '-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File "$INSTDIR\Start.ps1"' "$INSTDIR\goxlr-daemon.exe" 0 SW_SHOWMINIMIZED
 ${EndIf}
SectionEnd
Function un.onInit
 SetShellVarContext current
 SetRegView 64
 ${GetParameters} $0
 ClearErrors
 ${GetOptions} $0 "/SANDBOX" $1
 ${IfNot} ${Errors}
  StrCpy $Sandbox 1
 ${EndIf}
FunctionEnd
Section "Uninstall"
 ${If} $Sandbox != 1
  InitPluginsDir
  nsExec::ExecToStack '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$INSTDIR\Preflight.ps1" -Uninstall'
  Pop $0
  Pop $1
  ${If} $0 != 0
   MessageBox MB_ICONSTOP "$(Running)"
   SetErrorLevel 3
   Abort
  ${EndIf}
  Delete "$SMSTARTUP\SlopXLR.lnk"
  Delete "$DESKTOP\SlopXLR.lnk"
  Delete "$SMPROGRAMS\SlopXLR\SlopXLR.lnk"
  RMDir "$SMPROGRAMS\SlopXLR"
  DeleteRegKey HKCU "Software\Skolastico\GoXLR-Personalizado"
  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\GoXLR-Personalizado"
 ${EndIf}
 ; Delete only program files; user-data, config and AppData are preserved.
 Delete "$INSTDIR\goxlr-daemon.exe"
 Delete "$INSTDIR\goxlr-utility-ui.exe"
 Delete "$INSTDIR\goxlr-defaults.exe"
 Delete "$INSTDIR\SAAPI64.dll"
 Delete "$INSTDIR\nvdaControllerClient64.dll"
 Delete "$INSTDIR\Open.cmd"
 Delete "$INSTDIR\Start.ps1"
 Delete "$INSTDIR\Update.ps1"
 Delete "$INSTDIR\Preflight.ps1"
 Delete "$INSTDIR\release.json"
 Delete "$INSTDIR\update-public.xml"
 Delete "$INSTDIR\update-status.json"
 Delete "$INSTDIR\installed.marker"
 Delete "$INSTDIR\WebView2Setup.exe"
 Delete "$INSTDIR\LICENSE"
 Delete "$INSTDIR\LICENSE-3RD-PARTY"
 Delete "$INSTDIR\LICENSE-UI"
 Delete "$INSTDIR\README.txt"
 Delete "$INSTDIR\Uninstall.exe"
 RMDir "$INSTDIR"
SectionEnd
