cargo build --release --target wasm32-unknown-unknown && ^
wasm-bindgen --no-typescript --out-name foretold --out-dir wasm --target web target/wasm32-unknown-unknown/release/foretold.wasm
robocopy assets wasm/assets /xf *.blend /xd fonts /s
7z a wasm.zip ./wasm/*