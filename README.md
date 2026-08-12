# StellarHash 🚀

![Release](https://github.com/ProceduralDynamics/StellarHash/actions/workflows/ci.yml/badge.svg)
![Release](https://github.com/ProceduralDynamics/StellarHash/actions/workflows/release.yml/badge.svg)
![Release](https://github.com/ProceduralDynamics/StellarHash/actions/workflows/rename_issues.yml/badge.svg)

rename_issues
> A deterministic, procedurally generated 2D universe explorer built from scratch using Rust and the [Bevy Engine](https://bevyengine.org/).

<img width="1335" height="244" alt="image" src="https://github.com/user-attachments/assets/5e0837b1-ff7b-40ae-a665-315ea19c91f3" />

---
## 🚀 Overview
StellarHash utilizes a custom spatial hashing algorithm to generate an infinite, persistent universe.    

Every set of coordinates yields the exact same star system based on a global seed, ensuring deterministic exploration without the need to save massive amounts of data.    

The engine is highly optimized, featuring dynamic memory management, spatial grid filtering, and real-time Level of Detail (LOD) to maintain a smooth frame rate even with thousands of visible celestial bodies.


---
## Features
- Infinite Procedural Generation:
    Seamlessly explore a boundless universe. Stars are generated dynamically as you move the camera and cleaned up automatically to preserve memory.

- Realistic Stellar Classification:
    Stars are classified from `O` to `M` (Morgan-Keenan system), dictating their color, radius, mass, age, and planetary likelihood.

- Interactive Star Systems:
    Click on any generated star to reveal its planetary system. Planets follow Kepler-inspired orbital mechanics with real-time trigonometric animations.

- Dynamic Hover UI:
    Hover over any star to instantly intercept its telemetry data (Name, Mass, Age, Planet count).

- Deep Space Transmissions:
    A dynamic UI panel broadcasts random, real-world astrophysics facts while you explore.

- Highly Optimized:
    Engineered for maximum performance (60+ FPS) using spatial grid filtering, Level of Detail (LOD) rendering, and automated garbage collection.

- Custom WGSL Shaders:
    Leverages Bevy's `Material2d` pipeline with custom WebGPU Shading Language (WGSL) scripts. By offloading visual computations to the GPU (`plasma.wgsl`, `star.wgsl`), the engine renders high-fidelity, dynamic plasma effects for giant stars without compromising CPU performance.

---
## Installation & Build

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install)
- Bevy dependencies for Linux (e.g., `g++`, `pkg-config`, `libx11-dev`, `libasound2-dev`, `libudev-dev`)

### Running Locally
Clone the repository and run the project in **release mode** for optimal performance:

```bash
git clone https://github.com/ProceduralDynamics/StellarHash.git
```

```bash
cd StellarHash
```

```bash
cargo run --release
```

---

---
## Benchmarks
Given the procedural and deterministic nature of the universe, performance is a core focus of StellarHash. We maintain a suite of benchmarks to ensure the spatial hashing and generation algorithms remain lightning-fast and capable of sustaining 60+ FPS.

To run the benchmark suite locally on your machine:
```bash
cargo bench
```

---
## 🤝 Contributing
We welcome contributions! To keep the project organized, please follow these steps:

1. Open an issue **using the provided Issue Templates** (Bug Report or Feature Request) to discuss your idea.
2. Create a dedicated branch (`git switch -c feature/your-feature`).
3. Commit your changes.
4. Open a Pull Request against the `dev` branch.

---
## Project Architecture
* `main.rs`: Entry point and Bevy App configuration.
* `univers.rs`: Core procedural generation loop, spatial caching (`Local<T>`), LOD management, and garbage collection.
* `generation.rs`: Deterministic spatial hashing mathematics (bitwise operations).
* `astrophysique.rs`: Data models and procedural rules for star classification.
* `ui.rs`: Spatial grid raycasting, hover detection, and UI rendering.
* `camera.rs`: 2D movement and scaling logic.
* `lib.rs`: Exposes the internal modules of the project, defining the public API for the crate structure.

---
## Note

> If you are downloading the pre-compiled binary release, ensure the **assets/** and **fonts/** directories remain at the root level alongside the executable for the UI and text to load properly.

---

