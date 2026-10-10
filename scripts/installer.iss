; HFS Desktop 安裝程式（Inno Setup 6.3+）
; 由 build-release.ps1 編譯：ISCC /DAppVersion=x.y.z /DExeFile=<hfs-gui.exe> installer.iss
; 建捷徑交給安裝程式做：未簽章的 exe 自己建 .lnk 容易被防毒判成可疑行為

#define AppName "HFS Desktop"
#define ShortcutName "HFS 檔案分享"
#define ExeName "hfs-gui.exe"
; 第一次發佈後不能再改：升級與解除安裝靠它找到舊的安裝
#define AppId "{{35A894A4-B289-4404-837C-56AA999FA3E6}"

#ifndef AppVersion
  #error 請用 build-release.ps1 編譯（要傳 /DAppVersion=...）
#endif
#ifndef ExeFile
  #error 請用 build-release.ps1 編譯（要傳 /DExeFile=...）
#endif
#ifndef OutputDir
  #define OutputDir "..\dist"
#endif

[Setup]
AppId={#AppId}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher=hfs-desktop
AppPublisherURL=https://github.com/win81134679-dot/hfs-desktop
VersionInfoVersion={#AppVersion}
VersionInfoProductName={#AppName}
VersionInfoDescription={#AppName} 安裝程式
; 只裝給目前帳號：%LOCALAPPDATA%\Programs\HFS Desktop，不需要系統管理員
DefaultDirName={autopf}\{#AppName}
PrivilegesRequired=lowest
DisableDirPage=yes
DisableProgramGroupPage=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
OutputDir={#OutputDir}
OutputBaseFilename=HFS-Desktop-Setup-v{#AppVersion}
SetupIconFile=..\hfs-gui\assets\app.ico
UninstallDisplayIcon={app}\{#ExeName}
UninstallDisplayName={#AppName}
LicenseFile=..\LICENSE
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
; 程式開著時不硬關（硬關會留下沒結束的 hfs.exe），請使用者先自己結束
AppMutex=hfs-gui-single-instance
CloseApplications=no

[Languages]
Name: "cht"; MessagesFile: "compiler:Default.isl,ChineseTraditional.isl"

[Messages]
cht.SetupAppRunningError=HFS 檔案分享正在執行。%n%n請先在系統匣圖示按右鍵選「結束」，再按「確定」繼續；按「取消」離開安裝。
cht.UninstallAppRunningError=HFS 檔案分享正在執行。%n%n請先在系統匣圖示按右鍵選「結束」，再按「確定」繼續；按「取消」離開。

[Tasks]
Name: "desktopicon"; Description: "在桌面建立「{#ShortcutName}」捷徑"; GroupDescription: "捷徑："

[Files]
Source: "{#ExeFile}"; DestDir: "{app}"; DestName: "{#ExeName}"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#ShortcutName}"; Filename: "{app}\{#ExeName}"; WorkingDir: "{app}"
Name: "{autodesktop}\{#ShortcutName}"; Filename: "{app}\{#ExeName}"; WorkingDir: "{app}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#ExeName}"; Description: "立即啟動 {#ShortcutName}"; Flags: nowait postinstall skipifsilent

[Code]
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  // 程式在「設定」裡開過「開機自動啟動」會寫這個值，解除安裝時一併清掉
  if CurUninstallStep = usPostUninstall then
    RegDeleteValue(HKEY_CURRENT_USER, 'Software\Microsoft\Windows\CurrentVersion\Run', '{#ShortcutName}');
end;
