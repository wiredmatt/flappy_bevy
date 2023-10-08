# Flappy Bevy

## Run

```
cargo run
```

First compile will take a while, but afterwards, you'll get 0.1 - 2s compile times.

Check the `build` branch for wasm release setup, there you can run:

```
cargo build --release --target wasm32-unknown-unknown
~/.cargo/bin/wasm-bindgen --out-dir ./web/ --target web ./target/wasm32-unknown-unknown/release/flappy_bevy.wasm
```