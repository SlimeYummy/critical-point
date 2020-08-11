@echo off
if "%1" == "--release" (
    cargo build --release
    copy .\target\release\critical_point_gd.dll .\scene
) else (
    cargo build
    copy .\target\debug\critical_point_gd.dll .\scene
)
