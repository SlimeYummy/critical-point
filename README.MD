[![Build Status](https://travis-ci.org/FenQiDian/critical-point.svg?branch=master)](https://travis-ci.org/FenQiDian/critical-point)

# Critical Point

This project is **WIP**.

A logic engine for 3D ACT games.

## Features

- Data driven and configurable.
- Human friendly config files (base on YAML).
- Independent of game engine, support headless mode.
- Deterministic, support lock-step networking synchronize.

## Modules

- derive: rust derive macros.
- math: math library include vector, matrix and e.g.
- collide: deterministic collision detection (base on [ncollide](https://github.com/dimforge/ncollide)).
- compiler: config file preprocessor.
- core: core library.
  - script: a very small script engine, tiny and fast.
  - resource: resource manager.
  - physic: physic logic.
  - value: value logic.
- plugin_x: plugins for game engine.
