# Build
cargo build --target wasm32-unknown-unknown --release

# Generate bindgen outputs
mkdir -p html
wasm-bindgen target/wasm32-unknown-unknown/release/megachess.wasm --out-dir html --target web --no-typescript

# Shim to tie the thing together
sed -i "s/import \* as __wbg_star0 from 'env';//" html/megachess.js
sed -i "s/let wasm;/let wasm; export const set_wasm = (w) => wasm = w;/" html/megachess.js
sed -i "s/imports\['env'\] = __wbg_star0;/return imports.wbg\;/" html/megachess.js

# Create index from the HTML variable
echo "$HTML" > html/index.html