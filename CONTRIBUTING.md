# Contributing

Contributions are welcome. Please open an issue before starting a larger change
so its scope and user-facing behavior can be discussed first.

## Development setup

RemoteDeskRDP requires Node.js 20 or newer, a current Rust toolchain and Xcode
Command Line Tools.

```sh
npm ci
npm run tauri:dev
```

Run these checks before submitting a pull request:

```sh
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
```

Changes to the bundled FreeRDP or SDL sources must be supplied as patches in
`scripts/patches/` and documented in `docs/FREERDP_PATCHES.md`. Do not commit
local build output from `.build/`, `dist/` or `src-tauri/target/`.

By submitting a contribution, you agree that it may be distributed under the
project's MIT License. Third-party components retain their own licenses.

