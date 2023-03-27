# About
Cross-stitch / Rust v2 experiments!

# Requirements
- Install Rust crates
- Install wasm-pack: https://rustwasm.github.io/wasm-pack/installer/
- Install node: https://nodejs.org/en/download
  - Install `http-server`
    - `npm install -g http-server`
    - If required, run `Set-ExecutionPolicy -ExecutionPolicy RemoteSigned` to enable starting the local HTTP server

# Notes
- Building locally: `cargo build`
  - Launch with `cargo run`
- Building for web: `wasm-pack build --target web`
  - Launch with  `http-server -c-1`, opening with `http://127.0.0.1:8080/main.html`