# NoString Heir

> Someone left you Bitcoin. This app helps you claim it.

Mobile app for heirs to claim Bitcoin from [NoString](https://github.com/RenAndKiwi/nostring) inheritable vaults. Cross-platform (iOS + Android).

## How It Works

1. **Import** your vault backup (QR code or pasted JSON)
2. **Wait** for the timelock countdown to expire
3. **Claim** — paste any Bitcoin address, sign, done

The owner set everything up. You just need this app and your signing key.

## Architecture

- **React Native + Expo** — cross-platform mobile
- **Rust core via UniFFI** — all crypto runs in Rust (nostring-ccd, nostring-inherit)
- **No keys stored** — signing happens on your hardware wallet or is derived once from seed

## Development

```bash
# Install dependencies
npm install

# iOS
npx expo run:ios

# Android
npx expo run:android

# Build Rust FFI
cd rust && cargo build --release
```

## License

BSD-3-Clause
