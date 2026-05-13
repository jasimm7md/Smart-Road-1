# Smart Road

A **smart intersection simulation** for autonomous vehicles (AVs) without traffic lights. Built with **Rust** and **SDL2**.

## Requirements

- **Rust** (stable, `rustup`)
- **CMake** (for building SDL2 bundled with the project)
- A **C/C++ toolchain** that CMake can use (e.g. **MSVC** with “Desktop development with C++” on Windows, or **build-essential** on Linux)

**Why SDL2 (2.x) and not SDL3?** The [project objectives](objectives/README.md) require **sdl2** (the Rust crate for SDL 2.x). SDL3 is a different API.

### How SDL is set up in this repo

- The [`sdl2`](https://crates.io/crates/sdl2) crate is used with the **`bundled`** feature: SDL2 is **compiled from source** when you run `cargo build`. You do **not** need to download SDL2 `.lib` / `.dll` separately on Windows.
- **SDL2_ttf is not used.** The statistics screen draws text with the [`font8x8`](https://crates.io/crates/font8x8) bitmap font (no system fonts, no extra native libs).

### Windows

1. Install **Visual Studio Build Tools** or Visual Studio with **Desktop development with C++** (MSVC, CMake optional component is fine).
2. Ensure **`cmake`** is on your `PATH` (bundled with VS or install [CMake](https://cmake.org/download/)).
3. Build from the project root:

   ```powershell
   cd D:\path\to\smart-road
   cargo build
   ```

The file **`.cargo/config.toml`** sets `CMAKE_POLICY_VERSION_MINIMUM=3.5` so **CMake 4+** can configure the SDL2 sources shipped with `sdl2-sys` (avoids the common CMake compatibility error).

**Optional (advanced):** If you turn **off** `bundled` in `Cargo.toml`, you must link against an installed SDL2 (see [rust-sdl2 Windows](https://github.com/Rust-SDL2/rust-sdl2#windows-msvc)) and extend the **`LIB`** environment variable with your `...\lib\x64` folder. The script `scripts/set-windows-sdl-lib.ps1` is only for that workflow.

### Linux

```bash
sudo apt install build-essential cmake pkg-config \
  libasound2-dev libudev-dev libdbus-1-dev \
  libx11-dev libxext-dev libxrandr-dev libxcursor-dev libxi-dev \
  libxinerama-dev libxxf86vm-dev libgl1-mesa-dev
# If `cargo build` still fails in sdl2-sys, install any extra packages CMake asks for, or see:
# https://wiki.libsdl.org/SDL2/README-linux
```

### macOS

```bash
xcode-select --install   # if needed
brew install cmake
```

## Build and run

```bash
cargo build
cargo run
```

On the first build, compiling SDL2 can take **1–3 minutes**. Later builds are fast.

Statistics are printed in the **console** when you press Esc; a **stats window** also opens with the same information (bitmap text).

**Nothing moving / no cars?** Click the **game window** so it has keyboard focus, then use **arrow keys** or hold **R**. (If the terminal has focus, SDL won’t see the keys.) There was also a spawn bug: the first arrow spawn required ~0.8s of sim time—**that’s fixed** so the first press works immediately.

## Controls

| Key | Action |
|-----|--------|
| **Arrow Up** | Spawn vehicle from South (bottom) |
| **Arrow Down** | Spawn vehicle from North (top) |
| **Arrow Left** | Spawn vehicle from East (right) |
| **Arrow Right** | Spawn vehicle from West (left) |
| **R** (hold) | Continuously spawn random vehicles |
| **Esc** | End simulation and show statistics |

Spawn rate is limited so vehicles are not created on top of each other when you press the same key repeatedly.

## Features

- **Cross intersection** with **three lanes per approach** (r / s / l), **solid lane dividers**, **dashed center lines**, road edges, and **lane labels** aligned with the [objectives diagram](objectives/README.md).
- **Autonomous vehicles** with at least 3 velocity levels (stopped, slow, normal).
- **Safe distance** and **conflict resolution**: no collisions; vehicles slow or stop when needed.
- **Physics**: velocity, distance, and time tracked per vehicle; time through intersection measured from detection until exit.
- **Statistics** (Esc): max vehicles passed, max/min velocity, max/min time through intersection, close calls.

## Project structure

- `src/main.rs` – Entry point, game loop, input, rendering, stats window.
- `src/intersection.rs` – Intersection geometry, paths, conflict matrix.
- `src/vehicle.rs` – Vehicle state and physics.
- `src/controller.rs` – Smart controller (velocity decisions).
- `src/stats.rs` – Statistics collection.
- `src/simple_text.rs` – Bitmap text for the stats UI (no SDL2_ttf).
- `src/render.rs` – Road surface, lane lines, dashed centers, r/s/l labels.
- `objectives/` – Project requirements and audit checklist.
- `.cargo/config.toml` – Build environment for CMake / bundled SDL2.

## License

For use according to the 01-edu / Smart Road subject.
