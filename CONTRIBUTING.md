# Contributing

## Tech Stack

| Layer    | Tech                                              |
| -------- | ------------------------------------------------- |
| Frontend | Svelte 5 + SvelteKit, TypeScript, Tailwind CSS v4 |
| Backend  | Tauri 2 (Rust)                                    |
| Build    | Vite, pnpm                                        |
| AI       | Ollama (local), OpenAI, Google Gemini             |

## Getting Started

```bash
cd apps/hazardo-ui
pnpm install
pnpm dev
```

## Production Builds

Requires [Rust](https://rustup.rs/), [Node.js](https://nodejs.org/), [pnpm](https://pnpm.io/), and the [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/).

| Platform              | Command                  | Output               |
| --------------------- | ------------------------ | -------------------- |
| Windows               | `make build-windows`     | `.exe` / `.msi`      |
| macOS (Apple Silicon) | `make build-macos`       | `.app` / `.dmg`      |
| macOS (Intel)         | `make build-macos-intel` | `.app` / `.dmg`      |
| Linux                 | `make build-linux`       | `.deb` / `.AppImage` |
| Android               | `make build-android`     | `.apk`               |
| iOS                   | `make build-ios`         | `.ipa`               |

## CI/CD

GitHub Actions builds all platforms automatically.

- **Push to `main`/`dev`** — CI runs frontend checks + builds all platforms
- **Tag push (`v*.*.*`)** — Builds + publishes a GitHub Release with all artifacts

```bash
git tag v1.0.0
git push origin v1.0.0
```

## Release Packaging (local)

```bash
make release-android    # → releases/v1.0.0/hazardo-v1.0.0.apk
make release-windows    # → releases/v1.0.0/hazardo-v1.0.0-windows-setup.exe
make release-macos      # → releases/v1.0.0/hazardo-v1.0.0-macos-arm64.dmg
make release-linux      # → releases/v1.0.0/hazardo-v1.0.0-linux.AppImage
```
