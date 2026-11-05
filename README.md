![Static Badge](https://img.shields.io/badge/State-Active-brightgreen?style=plastic)

<div align="center"">
  <pre >
 ▄▄▄▄▄▄                                      
█▀██▀▀██                                     
  ██   ██       ▄        ▄▄             ▄    
  ██   ██ ██ ██ ████▄ ▄████ ▄█▀█▄ ▄███▄ ████▄
▄ ██   ██ ██ ██ ██ ██ ██ ██ ██▄█▀ ██ ██ ██ ██
▀██▀███▀ ▄▀██▀█▄██ ▀█▄▀████▄▀█▄▄▄▄▀███▀▄██ ▀█
                         ██                  
                       ▀▀▀                   
  </pre>
</div>

Welcome to **Dungeon**! This is an experimental 3D game engine and dungeon-crawler video game built completely from scratch using **Rust**. The project explores 3D graphics rendering, math, and game logic from the ground up without relying on heavy commercial game engines.


<img width="1024" height="768" alt="dungeon-rusted" src="https://github.com/user-attachments/assets/6bb0187a-8d5c-4179-a48a-f16b910eb184" />

## Key Features

* **Custom 3D Engine:** Native matrix mathematics, vector operations, and rendering pipeline built from scratch.
* **Software Pixel Buffer:** Seamless window management and direct pixel manipulation powered by `softbuffer` and `winit`.
* **Terminal UI Integration:** Real-time debugging tools, internal metrics, and console management utilizing `ratatui` and `crossterm`.
* **Asset & Config Management:** Optimized state loading and data management using `toml` configurations and `serde`.
* **Robust Error Handling:** Crystal-clear runtime diagnostics and crash reporting implemented with `color-eyre`.

## Tech Stack

The project leverages modern, low-level crates within the Rust ecosystem:
* **Rust (2021 Edition)**
* **Windowing & Graphics:** `winit`, `softbuffer`
* **TUI (Terminal User Interface):** `ratatui`, `crossterm`
* **Serialization:** `serde`, `toml`
* **Diagnostics:** `color-eyre`

## Prerequisites

To compile and run this project, you need the Rust toolchain installed. If you don't have it yet, you can get it from [rustup.rs](https://rustup.rs).

## Installation & Running

1. Clone the repository:
   ```bash
   git clone https://github.com
   cd dungeon
   ```

2. Run the project in development mode:
   ```bash
   cargo run
   ```

3. Run the project with performance optimizations (highly recommended for 3D rendering):
   ```bash
   cargo run --release
   ```

## Project Funding Model (Steam)

Dungeon follows a funding model inspired by open-source projects like **Krita**. 

* **The Code is Free:** The entire source code is hosted here on GitHub under the GPL v3 license. Anyone can clone, compile, and play it for free.
* **Support via Steam:** We offer pre-compiled, optimized packages on Steam for a small fee (€7-8). Purchasing the game on Steam is completely voluntary; it gives you the convenience of automatic updates, cloud saves, and directly supports the continuous development of this project.

## Contributing

Contributions are what make the open-source community such an amazing place to learn, inspire, and create. Please read our [CONTRIBUTING.md](CONTRIBUTING.md) to understand our workflow and license agreements before submitting any code.

## License

This project is licensed under the **GNU General Public License v3 (GPL v3)**. See the `LICENSE` file for details.
