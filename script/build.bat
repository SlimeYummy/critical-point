@echo off
if "%1" == "--release" (
    cargo build --release
    copy .\target\release\critical_point_gd.dll .\scene\native
    .\target\release\compiler.exe .\scene\critical_point\resource.yml .\scene\critical_point\id.yml
) else (
    cargo build
    copy .\target\debug\critical_point_gd.dll .\scene\native
    .\target\debug\compiler.exe .\scene\critical_point\resource.yml .\scene\critical_point\id.yml
)
