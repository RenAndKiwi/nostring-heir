# NoString Heir

> Someone left you Bitcoin. This app helps you claim it.

PWA for heirs to claim Bitcoin from [NoString](https://github.com/RenAndKiwi/nostring) inheritable vaults. Works on any device with a browser.

## How It Works

1. **Import** your vault backup (QR code or pasted JSON)
2. **Wait** for the timelock countdown to expire
3. **Claim** — paste any Bitcoin address, sign, done

The owner set everything up. You just need this app and your signing key.

## Architecture

- **SvelteKit PWA** — works offline, no app store needed
- **Rust → WASM** — all crypto runs in Rust compiled to WebAssembly
- **No keys stored** — signing happens externally (hardware wallet or one-time seed derivation)

## Development

```bash
# Build WASM bindings
cd rust && wasm-pack build --target web nostring-heir-ffi

# Install JS dependencies
npm install

# Dev server
npm run dev

# Production build
npm run build
```

## License

BSD-3-Clause
