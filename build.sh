PROJECT_NAME=bugchess

# Build
cargo build --target wasm32-unknown-unknown --release

# Generate bindgen outputs
mkdir -p html
wasm-bindgen target/wasm32-unknown-unknown/release/${PROJECT_NAME}.wasm --out-dir html --target web --no-typescript

# Shim to tie wasm-bindgen output together with macroquad's gl.js loader
# (see https://github.com/not-fl3/macroquad/issues/212)
JS=html/${PROJECT_NAME}.js

# 1. Remove all `import * as importN from "env"` lines (macroquad env is provided by gl.js)
sed -i.bak '/^import \* as .* from "env"$/d' "$JS"

# 2. Export a set_wasm function so the miniquad plugin can pass wasm_exports back to wasm-bindgen
sed -i.bak 's/let wasmModule, wasm;/let wasmModule, wasm; export const set_wasm = (w) => wasm = w;/' "$JS"

# 3. In __wbg_get_imports(), replace the return block to only return the wasm-bindgen imports
#    (drop all "env": importN entries, keep only "./bugchess_bg.js": import0)
sed -i.bak '/"env": import[0-9]/d' "$JS"

# 4. Short-circuit init functions so they return imports without loading WASM
#    (macroquad's load() in gl.js handles WASM instantiation)
sed -i.bak 's/const imports = __wbg_get_imports();/return __wbg_get_imports();/' "$JS"

# Clean up backup files
rm -f "$JS.bak"