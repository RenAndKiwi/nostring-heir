# NoString Heir App

Mobile app for heirs to claim inherited Bitcoin from NoString vaults.

**Stack:** Flutter + Rust FFI (via flutter_rust_bridge)

## Architecture

The heir app is the mirror image of the owner's desktop app:
- Owner creates Taproot vault, adds heirs, delivers VaultBackup
- Heir imports VaultBackup, monitors timelock, builds claim PSBT, signs, broadcasts

## Why Flutter + Rust?

- `bitcoin` crate (secp256k1) doesn't compile to WASM — C FFI fails
- Flutter + flutter_rust_bridge compiles Rust natively (ARM/x86) — no limitations
- Reuses existing tested Rust code (280+ tests in nostring workspace)
- Native iOS + Android from one codebase

## Status

Starting fresh. Previous SvelteKit PWA approach scrapped (WASM limitations).

## License

BSD-3-Clause
