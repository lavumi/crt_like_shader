# CRT-like Shader Renderer

A WebGPU-based CRT (Cathode Ray Tube) shader renderer written in Rust, featuring retro-style visual effects and real-time rendering capabilities. This project demonstrates advanced shader techniques and can be built for both native desktop and web platforms using WebAssembly.

## Features

- **CRT Visual Effects**: Authentic cathode ray tube simulation with scanlines, phosphor glow, and curvature
- **WebGPU Rendering**: Modern graphics API support for both native and web platforms
- **Real-time Performance**: Optimized shaders for smooth 60fps rendering
- **Cross-platform**: Runs on desktop (native) and web browsers (WASM)
- **Configurable**: TOML-based configuration system for easy customization

## Prerequisites

### 0. Rust Installation

Before you can build and run this project, you need to install Rust:

#### On macOS and Linux:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### On Windows:
Download and run the installer from [rustup.rs](https://rustup.rs/)

#### Verify Installation:
```bash
rustc --version
cargo --version
```

#### Additional Requirements for WASM:
```bash
# Install wasm-pack for building WebAssembly
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Add WASM target
rustup target add wasm32-unknown-unknown
```

## Building and Running

### 1. Native Build and Execution

For desktop/native execution:

```bash
# Clone the repository (if not already done)
git clone <repository-url>
cd crt_like_shader

# Build the project
cargo build

# Run the application
cargo run
```

#### Release Build (Optimized):
```bash
cargo build --release
cargo run --release
```

### 2. WebAssembly Build and Execution

For web browser execution:

#### Build WASM:
```bash
# Build the WebAssembly package
wasm-pack build --target web --out-dir pkg

# This will generate:
# - pkg/crt_like_js_renderer.js
# - pkg/crt_like_js_renderer_bg.wasm
# - pkg/crt_like_js_renderer.d.ts
# - pkg/package.json
```

#### Serve the Web Application:
```bash
# Install basic-http-server (recommended)
cargo install basic-http-server

# Serve the project directory
basic-http-server

# Or use Python's built-in server (alternative)
python3 -m http.server 8000

# Or use Node.js serve (alternative)
npx serve .
```

#### Access the Application:
Open your web browser and navigate to:
- `http://localhost:8000` (if using basic-http-server)
- `http://localhost:8000` (if using Python server)
- The port may vary depending on your server

## Project Structure

```
crt_like_shader/
├── src/                    # Rust source code
│   ├── main.rs            # Entry point for native builds
│   ├── lib.rs             # Library entry point for WASM
│   ├── renderer.rs        # WebGPU rendering logic
│   ├── buffer.rs          # Buffer management
│   ├── config.rs          # Configuration handling
│   └── resources.rs       # Resource management
├── res/                   # Resources and assets
│   ├── shader/           # WGSL shader files
│   │   ├── crt.wgsl      # Main CRT shader
│   │   ├── post_crt.wgsl # Post-processing effects
│   │   └── colour_tile.wgsl # Color tile shader
│   ├── chr.png           # Character sprite
│   └── game_config.toml  # Game configuration
├── pkg/                   # Generated WASM package (after wasm-pack build)
├── index.html            # Web application entry point
├── wasm-binds.js         # WASM binding utilities
├── build.rs              # Build script for resource copying
└── Cargo.toml           # Rust project configuration
```

## Configuration

The project uses TOML configuration files for easy customization:

- **`res/game_config.toml`**: Main game and rendering configuration
- Modify shader parameters in the respective `.wgsl` files
- Adjust window settings in the source code

## Development

### Adding New Shaders:
1. Create your `.wgsl` file in `res/shader/`
2. Add the shader loading logic in `src/renderer.rs`
3. Rebuild the project

### Modifying CRT Effects:
- Edit `res/shader/crt.wgsl` for main CRT effects
- Edit `res/shader/post_crt.wgsl` for post-processing
- Adjust parameters in `res/game_config.toml`

## Troubleshooting

### Common Issues:

1. **WebGL Context Lost**: Refresh the browser page
2. **WASM Loading Failed**: Ensure you're serving over HTTP/HTTPS, not file://
3. **Build Errors**: Make sure all dependencies are installed and Rust is up to date
4. **Performance Issues**: Try the release build (`cargo build --release`)

### Browser Compatibility:
- Chrome/Chromium (recommended)
- Firefox
- Safari (limited WebGPU support)
- Edge

## Dependencies

### Native Dependencies:
- `wgpu`: WebGPU implementation
- `winit`: Window management
- `image`: Image processing
- `cgmath`: Math utilities
- `toml`: Configuration parsing

### WASM Dependencies:
- `wasm-bindgen`: Rust-WASM bindings
- `web-sys`: Web API bindings
- `js-sys`: JavaScript API bindings

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test both native and WASM builds
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- WebGPU working group for the modern graphics API
- WGPU team for the excellent Rust WebGPU implementation
- The Rust community for amazing tooling and ecosystem
