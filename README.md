# Newton's Cannon

Newton's cannonball was a thought experiment Isaac Newton used to hypothesize that the force of gravity was universal

---

## Online

You can test the app online on github pages

- https://gbpagano.github.io/newton_cannon/

## Binary

To run locally just clone the project and run it with cargo:

```bash
cargo run --release --features bevy/dynamic_linking
```

## WebAssembly

To build in wasm and run in the browser first install the dependencies:

```bash
rustup target install wasm32-unknown-unknown
cargo install wasm-server-runner
```

And run in with cargo:

```bash
cargo run --release --target wasm32-unknown-unknown
```

Or use [Trunk](https://trunkrs.dev/) to build and bundle with js snippets and source html file

First install the dependencies:

```bash
cargo install --locked trunk wasm-bindgen-cli
```

And follow the next commands:

```bash
touch index.html  # put yout css configs here
trunk build --release -d wasm index.html
cp -r assets wasm/assets
```

Done, run a local server in the wasm directory:

```bash
python -m http.server -d wasm
```

## Usage

- Spacebar | Enter ->  Create a new ball

- Arrow Keys -> Changes the initial velocity of the next ball

- Mouse -> Controls the camera


