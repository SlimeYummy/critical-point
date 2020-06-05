## Build Dependence

### Bullet3

Install cmake.

Update bullet3 submodule in the project.

```
git clone https://github.com/bulletphysics/bullet3.git
cd bullet3
cmake .
cmake -DBUILD_PYBULLET=OFF -DUSE_DOUBLE_PRECISION=OFF -DCMAKE_BUILD_TYPE=Release .
```

You can find static librarys in lib folder, headers in src folder.

### Godot

Read godot's official docs first (https://docs.godotengine.org/en/latest/development/compiling/).

Install Python and SCons PyWin32 (`pip install SCons`).

Compile godot, print all static library.

```
scons arch=x64 p=windows vsproj=yes target=release tools=no gdscript=no disable_advanced_gui=yes -j8
find . -path "*.lib"
```
