#define MyAppName "SunSynk Tray"
#define MyAppExeName "SunSynkTrayWin.exe"

#ifndef MyAppVersion
  #define MyAppVersion "0.0.0"
#endif

#ifndef MyPublishDir
  #define MyPublishDir "..\\bin\\Release\\net8.0-windows\\win-x64\\publish"
#endif

#ifndef MyOutputDir
  #define MyOutputDir "..\\..\\artifacts\\installer"
#endif

[Setup]
AppId={{6C6ACDED-0F1B-43A0-963D-00CF7D2C9876}}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher=sunsynktray
AppPublisherURL=https://github.com/leonjza/sunsynktray
DefaultDirName={autopf64}\{#MyAppName}
DefaultGroupName={#MyAppName}
OutputBaseFilename=SunSynkTrayWinSetup
OutputDir={#MyOutputDir}
ArchitecturesInstallIn64BitMode=x64
DisableDirPage=no
DisableProgramGroupPage=no

[Files]
Source: "{#MyPublishDir}\\*"; DestDir: "{app}"; Flags: recursesubdirs ignoreversion

[Icons]
Name: "{group}\\{#MyAppName}"; Filename: "{app}\\{#MyAppExeName}"
Name: "{commondesktop}\\{#MyAppName}"; Filename: "{app}\\{#MyAppExeName}"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "Create a &desktop icon";
