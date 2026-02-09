; AirDB Windows Installer
; NSIS Script with Admin Elevation and PATH Integration

!include "MUI2.nsh"
!include "FileFunc.nsh"
!include "EnvVarUpdate.nsh"

; Installer attributes
Name "AirDB"
OutFile "airdb-${VERSION}-windows-x64-setup.exe"
InstallDir "$PROGRAMFILES64\AirDB"
InstallDirRegKey HKLM "Software\AirDB" "InstallPath"
RequestExecutionLevel admin

; Version info
VIProductVersion "${VERSION}.0"
VIAddVersionKey "ProductName" "AirDB"
VIAddVersionKey "CompanyName" "AirDB"
VIAddVersionKey "FileDescription" "AirDB Installer"
VIAddVersionKey "FileVersion" "${VERSION}"
VIAddVersionKey "ProductVersion" "${VERSION}"
VIAddVersionKey "LegalCopyright" "MIT License"

; Modern UI settings
!define MUI_ABORTWARNING
!define MUI_ICON "..\..\src-tauri\icons\icon.ico"
!define MUI_UNICON "..\..\src-tauri\icons\icon.ico"

; Pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "..\..\LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

; Installation section
Section "Install"
    SetOutPath "$INSTDIR"
    
    ; Install main files
    File "bin\airdb.exe"
    File "bin\airdb-desktop.exe"
    File "bin\airdb-bootstrap.exe"
    
    ; Create bin directory and copy CLI there too
    CreateDirectory "$INSTDIR\bin"
    CopyFiles "$INSTDIR\airdb.exe" "$INSTDIR\bin\airdb.exe"
    CopyFiles "$INSTDIR\airdb-desktop.exe" "$INSTDIR\bin\airdb-desktop.exe"
    CopyFiles "$INSTDIR\airdb-bootstrap.exe" "$INSTDIR\bin\airdb-bootstrap.exe"
    
    ; Create versions directory for updater
    CreateDirectory "$INSTDIR\versions"
    CreateDirectory "$INSTDIR\versions\current"
    CopyFiles "$INSTDIR\airdb.exe" "$INSTDIR\versions\current\airdb-cli.exe"
    CopyFiles "$INSTDIR\airdb-desktop.exe" "$INSTDIR\versions\current\airdb-desktop.exe"
    CopyFiles "$INSTDIR\airdb-bootstrap.exe" "$INSTDIR\versions\current\airdb-bootstrap.exe"
    
    ; Create initial state.json
    FileOpen $0 "$INSTDIR\state.json" w
    FileWrite $0 '{"current_version":"${VERSION}","last_good_version":"${VERSION}","pending_version":null,"update_channel":"stable","last_check":null,"status":"idle","failed_boot_count":0,"max_failed_boots":3}'
    FileClose $0
    
    ; Add to system PATH
    ${EnvVarUpdate} $0 "PATH" "A" "HKLM" "$INSTDIR\bin"
    
    ; Create Start Menu shortcuts
    CreateDirectory "$SMPROGRAMS\AirDB"
    CreateShortcut "$SMPROGRAMS\AirDB\AirDB.lnk" "$INSTDIR\airdb-desktop.exe" "" "$INSTDIR\airdb-desktop.exe" 0
    CreateShortcut "$SMPROGRAMS\AirDB\Uninstall.lnk" "$INSTDIR\uninstall.exe" "" "$INSTDIR\uninstall.exe" 0
    
    ; Create Desktop shortcut
    CreateShortcut "$DESKTOP\AirDB.lnk" "$INSTDIR\airdb-desktop.exe" "" "$INSTDIR\airdb-desktop.exe" 0
    
    ; Write registry keys for uninstaller
    WriteRegStr HKLM "Software\AirDB" "InstallPath" "$INSTDIR"
    WriteRegStr HKLM "Software\AirDB" "Version" "${VERSION}"
    
    ; Add/Remove Programs entry
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\AirDB" "DisplayName" "AirDB"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\AirDB" "DisplayVersion" "${VERSION}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\AirDB" "Publisher" "AirDB"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\AirDB" "UninstallString" "$INSTDIR\uninstall.exe"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\AirDB" "DisplayIcon" "$INSTDIR\airdb-desktop.exe"
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\AirDB" "NoModify" 1
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\AirDB" "NoRepair" 1
    
    ; Get installed size
    ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
    IntFmt $0 "0x%08X" $0
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\AirDB" "EstimatedSize" "$0"
    
    ; Create uninstaller
    WriteUninstaller "$INSTDIR\uninstall.exe"
SectionEnd

; Uninstallation section
Section "Uninstall"
    ; Remove from PATH
    ${un.EnvVarUpdate} $0 "PATH" "R" "HKLM" "$INSTDIR\bin"
    
    ; Remove shortcuts
    Delete "$DESKTOP\AirDB.lnk"
    Delete "$SMPROGRAMS\AirDB\AirDB.lnk"
    Delete "$SMPROGRAMS\AirDB\Uninstall.lnk"
    RMDir "$SMPROGRAMS\AirDB"
    
    ; Remove files
    Delete "$INSTDIR\airdb.exe"
    Delete "$INSTDIR\airdb-desktop.exe"
    Delete "$INSTDIR\airdb-bootstrap.exe"
    Delete "$INSTDIR\state.json"
    Delete "$INSTDIR\uninstall.exe"
    
    ; Remove directories
    RMDir /r "$INSTDIR\bin"
    RMDir /r "$INSTDIR\versions"
    RMDir "$INSTDIR"
    
    ; Remove registry keys
    DeleteRegKey HKLM "Software\AirDB"
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\AirDB"
SectionEnd

; Functions
Function .onInit
    ; Check for admin rights
    UserInfo::GetAccountType
    Pop $0
    ${If} $0 != "admin"
        MessageBox MB_ICONSTOP "Administrator rights are required to install AirDB."
        Abort
    ${EndIf}
FunctionEnd
