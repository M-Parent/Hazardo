# Releases

This folder contains the built artifacts for each version of Hazardo.

## Structure

```
releases/
├── v1.0.0/
│   ├── hazardo-v1.0.0.apk              # Android
│   ├── hazardo-v1.0.0-windows-setup.exe # Windows
│   ├── hazardo-v1.0.0-windows.msi      # Windows (MSI)
│   ├── hazardo-v1.0.0-macos-arm64.dmg  # macOS (Apple Silicon)
│   ├── hazardo-v1.0.0-linux.deb        # Linux (Debian)
│   └── hazardo-v1.0.0-linux.AppImage   # Linux (AppImage)
├── v1.1.0/
│   └── ...
└── ...
```

## How to add a release

Use the Makefile commands after building:

```bash
make release-android
make release-windows
make release-macos
make release-linux
```

Or manually:

1. Create a new folder with the version number: `vX.Y.Z/`
2. Place the built artifacts inside with the naming convention: `hazardo-vX.Y.Z-<platform>.<ext>`
3. Commit and push
