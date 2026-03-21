# Hazardo

![CI](https://github.com/pmusic62/Hazardo/actions/workflows/ci.yml/badge.svg)
![Release](https://github.com/pmusic62/Hazardo/actions/workflows/release.yml/badge.svg)

**Random Activity Picker** — An app that helps you decide what to do when you have no idea.

Roll the dice, and Hazardo picks a random activity from your personalized vault.

## Features

- **Roll & Pick** — Choose your time (AM/PM/Night), vibe (Friend/Date/Family), category, and Hazardo randomly picks an activity for you
- **Vault** — Create and manage your categories and activities (50+ default categories)
- **History** — Browse your past picks with notes, photos, location and weather
- **AI Chatbot** — Get suggestions from an AI (Ollama, OpenAI, Gemini)
- **Multi-user** — Multiple profiles on the same device
- **Bilingual** — French & English
- **Theming** — Light & Dark mode

## Tech Stack

| Layer    | Tech                                              |
| -------- | ------------------------------------------------- |
| Frontend | Svelte 5 + SvelteKit, TypeScript, Tailwind CSS v4 |
| Backend  | Tauri 2 (Rust)                                    |
| Build    | Vite, pnpm                                        |
| AI       | Ollama (local), OpenAI, Google Gemini             |

## Getting Started

```bash
# Install dependencies
cd apps/hazardo-ui
pnpm install

# Dev mode
pnpm dev
```

## Production Builds

Requires [Rust](https://rustup.rs/), [Node.js](https://nodejs.org/), [pnpm](https://pnpm.io/), and the [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/) for your platform.

| Platform              | Command                  | Output               |
| --------------------- | ------------------------ | -------------------- |
| Windows               | `make build-windows`     | `.exe` / `.msi`      |
| macOS (Apple Silicon) | `make build-macos`       | `.app` / `.dmg`      |
| macOS (Intel)         | `make build-macos-intel` | `.app` / `.dmg`      |
| Linux                 | `make build-linux`       | `.deb` / `.AppImage` |
| Android               | `make build-android`     | `.apk`               |
| iOS                   | `make build-ios`         | `.ipa`               |

### Release packaging

After building, copy artifacts into the versioned `releases/` folder:

```bash
make release-android    # → releases/v1.0.0/hazardo-v1.0.0.apk
make release-windows    # → releases/v1.0.0/hazardo-v1.0.0-windows-setup.exe
make release-macos      # → releases/v1.0.0/hazardo-v1.0.0-macos-arm64.dmg
make release-linux      # → releases/v1.0.0/hazardo-v1.0.0-linux.AppImage
```

The version is read automatically from `package.json`.

## CI/CD

GitHub Actions builds all platforms automatically.

**On every push to `main`/`dev`** — the [CI workflow](.github/workflows/ci.yml) runs:

- Frontend typecheck (Svelte)
- Desktop builds: Windows, macOS (ARM + Intel), Linux
- Android APK build

**On tag push (`v*.*.*`)** — the [Release workflow](.github/workflows/release.yml) builds everything and publishes a GitHub Release with all artifacts:

```bash
# Create a release
git tag v1.0.0
git push origin v1.0.0
# → GitHub Actions builds all platforms
# → Creates a GitHub Release with .exe, .msi, .dmg, .deb, .AppImage, .apk
```

Downloads will be available on the [Releases page](https://github.com/pmusic62/Hazardo/releases).

## Releases

Built artifacts are available in the [`releases/`](releases/) folder.

Each version has its own folder:

```
releases/
├── v1.0.0/
│   ├── hazardo-v1.0.0.apk
│   ├── hazardo-v1.0.0-windows-setup.exe
│   ├── hazardo-v1.0.0-macos-arm64.dmg
│   └── hazardo-v1.0.0-linux.AppImage
├── v1.1.0/
│   └── ...
└── ...
```

## License

MIT
