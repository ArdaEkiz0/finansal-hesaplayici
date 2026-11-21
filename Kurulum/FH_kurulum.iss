; Finansal Hesaplayici — Inno Setup 6 kurulum betiği (per-user)
; Referans: eski "kdv kontrol" Inno kalıbı (kopyalanmadı, sıfırdan yazıldı).
; Sürümün tek kaynağı Cargo workspace sürümüdür. CI, etiketten okuduğu değeri
; /DSurum=$v ile geçirir; yerel derlemede aşağıdaki varsayılan kullanılır.

#define MyAppName "Finansal Hesaplayici"
#ifndef Surum
  #define Surum "5.7.6"
#endif
#define MyAppVersion Surum
#define MyAppPublisher "Arda Mehmet Ekiz"
#define MyAppCopyright "Copyright © 2026 Arda Mehmet Ekiz"
#define MyAppExe "Finansal Hesaplayici.exe"
#define MyAppId "{{C1D2E3F4-5A6B-7C8D-9E0F-1A2B3C4D5E6F}}"

#ifndef SetupIcon
  #if FileExists("..\build\icon.ico")
    #define SetupIcon "..\build\icon.ico"
  #endif
#endif

[Setup]
AppId={#MyAppId}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
VersionInfoCompany={#MyAppPublisher}
VersionInfoCopyright={#MyAppCopyright}
VersionInfoDescription={#MyAppName} kurulum
VersionInfoProductName={#MyAppName}
VersionInfoProductVersion={#MyAppVersion}
VersionInfoVersion={#MyAppVersion}
; lowest = yönetici izni gerekmez, kurulum kullanıcı başınadır.
PrivilegesRequired=lowest
DefaultDirName={localappdata}\Programs\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
#ifdef SetupIcon
SetupIconFile={#SetupIcon}
#endif
UninstallDisplayIcon={app}\{#MyAppExe}
UninstallDisplayName={#MyAppName} {#MyAppVersion}
CreateUninstallRegKey=yes
Compression=lzma2
SolidCompression=yes
OutputDir=..\target\installer
OutputBaseFilename=Finansal-Hesaplayici-{#MyAppVersion}-setup
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible

[Languages]
Name: "turkish"; MessagesFile: "compiler:Languages\Turkish.isl"

[Tasks]
Name: "desktopicon"; Description: "Masaüstü kısayolu oluştur"; GroupDescription: "Ek kısayollar:"

[Files]
; Tek dosya: release derlemesi. ignoreversion ile üzerine güvenle yazılır.
Source: "..\target\release\Finansal_Hesaplayici.exe"; DestDir: "{app}"; DestName: "{#MyAppExe}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExe}"; IconFilename: "{app}\{#MyAppExe}"
Name: "{group}\{#MyAppName} Kaldır"; Filename: "{uninstallexe}"
Name: "{userdesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExe}"; IconFilename: "{app}\{#MyAppExe}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExe}"; Description: "{cm:LaunchProgram,{#MyAppName}}"; Flags: nowait postinstall skipifsilent
