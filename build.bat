@echo off

set DEST_CP=..\critical-point-u3d\Assets\CriticalPoint
set DEST_CS=..\critical-point-u3d\Assets\Scripts

if "%1" == "--release" (
    cargo build --release
    copy .\target\release\critical_point_u3d.dll %DEST_CP%
    .\target\release\compiler.exe %DEST_CP% resource.yml id.yml
) else (
    cargo build
    copy .\target\debug\critical_point_u3d.dll %DEST_CP%
    .\target\debug\compiler.exe %DEST_CP% resource.yml id.yml
)

copy .\target\FFIData.cs %DEST_CS%
copy .\target\FFIAgent.cs %DEST_CS%
copy .\target\FFIAutoGen.cs %DEST_CS%
