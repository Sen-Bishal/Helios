# H E L I O S

### HELIOS is a high-performance, deterministic simulation engine built in Rust to emulate the physics and mechatronics of High-Numerical Aperture (High-NA) EUV lithography systems. I saw this one youtube viedeo about the ridiculous engineering behind ASML's EUV Lithography machine. Got inspired. So much so that i read the book about ASML's design philosophy. What better way to learn intricate engineering that designing a system from scratch in rust right?
Feel free to watch the video btw -> https://www.youtube.com/watch?v=MiUHjLxm3V0
i got some good brainrot content as well if that's more up your alley <3

## System Architecture

The system is built on three architectural pillars to ensure the sub nanometer precision required for 13.5nm wavelength simulation.

### 1. Fixed-Point Spatial Grid

To avoid floating-point drift, the simulation uses a custom fixed-point coordinate system.

- **Base Unit**: 1 Picometer (pm)
- **Range**: Supports scales from 10⁻¹² m to 10¹ m
- **Implementation**: Wrapped `i128` types to prevent loss of precision over millions of simulation ticks

### 2. High-Frequency Microkernel

The EUV source operates at 50kHz. The simulation kernel uses a synchronous heartbeat to maintain determinism.

- **Tick Rate**: 1 microsecond
- **Concurrency**: Data parallel systems (Rayon) handle ray-tracing and particle dynamics across CPU threads without breaking determinism

### 3. Data-Oriented Design (ECS)

The simulator treats the machine as a collection of entities.

- **Entities**: Droplets, photons, mirrors, and sensors
- **Systems**: Discrete logic gates for Kinematics, Thermal Modeling, and Ray Intersections

## Development Roadmap

| Phase   | Focus                                                                                      | Status    |
|---------|--------------------------------------------------------------------------------------------|-----------|
| Phase 1 | Core Foundation: Fixed-point math, ECS structure, and microsecond heartbeat                | Completed |
| Phase 2 | The Source: Tin droplet injection (50kHz) and dual-pulse laser plasma generation           | Upcoming  |
| Phase 3 | Optics: Bragg reflector physics (Mo/Si stacks) and thermal absorption modeling             | Planned   |
| Phase 4 | Mechatronics: 20G stage acceleration and PID feedback control loops                        | Planned   |
| Phase 5 | Exposure: Reticle pattern projection and wafer dose accumulation                           | Planned   |

## Technical Implementation Tasks

| Task            | Description                                                          | Priority |
|-----------------|----------------------------------------------------------------------|----------|
| `units.rs`      | Finalize overflow checks for i128 picometer arithmetic               | High     |
| `kernel.rs`     | Implement the fixed-timestep loop with precise telemetry             | High     |
| `source.rs`     | Define the state machine for tin droplet transitions                 | Medium   |
| `optics.rs`     | Research and implement reflection coefficients for 13.5nm light      | Medium   |
| `benchmarking`  | Establish a baseline for rays-per-second throughput                  | Low      |

## System Requirements

- Rust (whatever version).
- Multi core CPU (recommended: 8+ cores for optimal parallelization) or a broke ahh CPU.
- Minimum 16GB RAM for large scale simulations, would recommend 32 gigs later but y'all know RAM sticks are literal gold rn.

## Design Principles

The simulator adheres to strict determinism requirements necessary for reproducible lithography research. All physical quantities are represented using fixed-point arithmetic to eliminate cumulative floating point errors that would otherwise compromise simulation accuracy at the picometer scale.

The microsecond resolution kernel ensures that all temporal phenomena from laser pulse timing to stage positioning—are modeled with sufficient granularity to capture the dynamics of a 50kHz operating frequency while maintaining computational efficiency through carefully designed parallel execution strategies.
