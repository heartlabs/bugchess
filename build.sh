PROJECT_NAME=bugchess

# Build
cargo build --target wasm32-unknown-unknown --release

# Generate bindgen outputs
mkdir -p html
wasm-bindgen target/wasm32-unknown-unknown/release/${PROJECT_NAME}.wasm --out-dir html --target web --no-typescript

# Shim to tie the thing together
sed -i.bak "s/import \* as __wbg_star0 from 'env';//" html/${PROJECT_NAME}.js
sed -i.bak "s/let wasm;/let wasm; export const set_wasm = (w) => wasm = w;/" html/${PROJECT_NAME}.js
sed -i.bak "s/imports\['env'\] = __wbg_star0;/return imports.wbg\;/" html/${PROJECT_NAME}.js
sed -i.bak "s/const imports = __wbg_get_imports();/return __wbg_get_imports();/" html/${PROJECT_NAME}.js