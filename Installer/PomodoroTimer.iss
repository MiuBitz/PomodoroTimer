; Inno Setup installer for Pomodoro Timer
#define MyAppName "Pomodoro Timer"
#define MyAppVersion "1.0.0"
#define MyPublisher "Pomodoro Timer"
#define MyExeName "PomodoroTimer.exe"

[Setup]
AppId={{7C2D3B80-0E32-4EAF-8D6A-5F3E4F9C2A11}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyPublisher}
DefaultDirName={autopf}\Pomodoro Timer
DefaultGroupName={#MyAppName}
OutputDir=Output
OutputBaseFilename=PomodoroTimerSetup
Compression=lzma
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
SetupIconFile=..\Assets\pomodoro.ico
UninstallDisplayIcon={app}\{#MyExeName}

[Files]
Source: "..\dist\PomodoroTimer.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyExeName}"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional icons:"

[Run]
Filename: "{app}\{#MyExeName}"; Description: "Launch Pomodoro Timer"; Flags: nowait postinstall skipifsilent

