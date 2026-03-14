# heroes-of-the-storm-on-rust

> **⚠️ Archived — This project is no longer maintained. Development has been permanently suspended.**

An ambitious attempt to rebuild a Heroes of the Storm-inspired game engine from scratch in Rust — without using any existing game engine. The project served as a deep learning experience in graphics programming, ECS architecture, networking, and engine design. Development was eventually abandoned in favor of [Bevy](https://bevyengine.org/), where a new project is now underway.

---

## What This Was

The goal was to recreate the feel of Heroes of the Storm using Rust, while addressing what the author perceived as architectural limitations in the original StarCraft 2 engine — specifically around character representation and game state management.

The most interesting design goal: **a time-reversal system**. Every significant piece of game state was wrapped in a `Reversible<T>` type that maintains a timestamped history of changes, allowing the game world to be "rewound" to any point in the recent past. A working demo of this mechanic was achieved.

After that, the project grew into an unmanageable scope and was abandoned.

---

## Architecture

The project is split into two binaries and a shared library:

```
heroes-of-the-storm-on-rust/
├── src/           (shared library)
│   ├── ecs/       (ECS components: Position, Rotation, Destination, Camera, Input...)
│   ├── heroes/    (hero definitions — only Tracer stubbed out)
│   ├── support/   (rendering: glium window/loop, font atlas, image renderer, GLTF models)
│   └── prelude.rs (fat re-export of everything)
│
├── client/main.rs (connects to server, renders world via glium)
└── server/main.rs (accepts TCP connections, runs bevy_ecs schedule)
```

### Key Systems

**`Reversible<T>`** — a generic wrapper around any value that records every change with a timestamp. Supports `get_value_seconds_ago(duration)`, `values_during(duration)`, and implements `AddAssign`, `SubAssign`, `MulAssign`, `DivAssign` — all of which also record history. This is the core of the time-rewind mechanic.

**`Position`, `Rotation`, `Destination`** — ECS components built on top of `Reversible<f32>` using a macro (`struct_with_vector!`). Every coordinate mutation is automatically tracked.

**Rendering** — Built on `glium` + `glutin` + `winit`. Includes a custom font atlas renderer (using `fontdue`), a texture/image renderer with a builder API, GLTF model loading, wavefront OBJ loading, and a shadow-map-ready `ModelData` type with a fluent transform builder.

**Networking** — Simple TCP client/server using `tokio`. The server runs a `bevy_ecs` `Schedule` in a loop (150ms tick). Clients send `ClientMessage` and receive `ServerMessage`, both serialized with `bincode`. A full world snapshot is sent on connect.

**Font Atlas** — Custom GPU-side glyph atlas built with `fontdue`. Supports both screen-space and world-space text rendering via separate shader paths.

---

## Dependencies (highlights)

| Crate | Role |
|-------|------|
| `glium` | OpenGL abstraction |
| `winit` + `glutin` + `glutin-winit` | Window & GL context |
| `cgmath` | Math (vectors, matrices) |
| `bevy_ecs` + `bevy_time` | Entity-Component-System |
| `tokio` | Async runtime (client + server) |
| `bincode` + `serde` | Network serialization |
| `fontdue` | Font rasterization for atlas |
| `gltf` | 3D model loading |
| `tracing` + `tracing-subscriber` | Structured logging |
| `paste` | Macro helper for identifier generation |
| `thiserror` + `anyhow` | Error handling |

---

## What Was Achieved

- Custom OpenGL rendering loop with delta-time, fullscreen window, and resize handling
- Font atlas with screen-space and world-space text rendering
- Image/sprite renderer with builder API, tint, rotation, and custom shaders
- Wavefront OBJ and GLTF model loading
- Camera with cursor-to-world ray projection
- Keyboard and mouse input abstraction
- TCP client/server with async read/write and exponential reconnect backoff
- Full world snapshot on player connect
- ECS-driven server loop (bevy_ecs without Bevy's app framework)
- `Reversible<T>` — time-tracked values with historical queries
- Working time-rewind proof of concept
- Dual logging (file + stdout) with `tracing`

## What Was Not Achieved

- Actual gameplay — no abilities, no combat loop, no win conditions
- More than one hero (Tracer was the only stub)
- Delta-state networking (full snapshots only)
- Proper game loop timing on the server side
- Most of the engine scope that was originally planned

---

## Why Archived

The project grew beyond what one person could manage alone. After successfully implementing the time-reversal prototype — which was the most interesting research goal — the remaining work felt like building an entire game engine with no end in sight.

The switch to Bevy solved the engine problem. A new project continues there.

---

## A Note on the Name and Assets

This project was *inspired by* Heroes of the Storm, not a copy of it. No game assets from Blizzard are included in this repository. As long as the `assets/` folder (containing any third-party art, models, or audio) is not distributed, the codebase itself represents original work. The name can be changed freely — the underlying engine ideas are entirely independent of any specific IP.

---

*Solo project. Built to learn. Learned a lot.*