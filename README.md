## Build Dependence

Read godot's official docs first (https://docs.godotengine.org/en/latest/development/compiling/).

Install Python and SCons PyWin32 (`pip install SCons`).

Compile godot, print all static library.

```
scons arch=x64 p=windows vsproj=yes target=release tools=no gdscript=no disable_advanced_gui=yes -j8
find . -path ".lib"
```
