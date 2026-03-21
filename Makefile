# ─── Hazardo Build System ─────────────────────────────────────
# Prerequisites:
#   - Node.js, pnpm, Rust (rustup), Tauri CLI
#   - Platform-specific: Xcode (iOS/macOS), MSVC (Windows), system libs (Linux)
#   - See: https://v2.tauri.app/start/prerequisites/
# ──────────────────────────────────────────────────────────────

APP_DIR     = apps/hazardo-ui
VERSION    := $(shell node -p "require('./$(APP_DIR)/package.json').version")

# ─── Dev ──────────────────────────────────────────────────────

.PHONY: install dev

install:
	cd $(APP_DIR) && pnpm install

dev:
	cd $(APP_DIR) && pnpm tauri dev

# ─── Production Builds ───────────────────────────────────────

# Windows (.exe / .msi)
.PHONY: build-windows
build-windows:
	cd $(APP_DIR) && pnpm tauri build --target x86_64-pc-windows-msvc
	@echo "✅ Windows build done → $(APP_DIR)/src-tauri/target/release/bundle/"

# macOS (.app / .dmg)
.PHONY: build-macos
build-macos:
	cd $(APP_DIR) && pnpm tauri build --target aarch64-apple-darwin
	@echo "✅ macOS (Apple Silicon) build done → $(APP_DIR)/src-tauri/target/aarch64-apple-darwin/release/bundle/"

.PHONY: build-macos-intel
build-macos-intel:
	cd $(APP_DIR) && pnpm tauri build --target x86_64-apple-darwin
	@echo "✅ macOS (Intel) build done → $(APP_DIR)/src-tauri/target/x86_64-apple-darwin/release/bundle/"

# Linux (.deb / .AppImage)
.PHONY: build-linux
build-linux:
	cd $(APP_DIR) && pnpm tauri build --target x86_64-unknown-linux-gnu
	@echo "✅ Linux build done → $(APP_DIR)/src-tauri/target/release/bundle/"

# Android (.apk)
.PHONY: build-android
build-android:
	cd $(APP_DIR) && pnpm tauri android build --apk
	@echo "✅ Android APK build done"

# iOS (.ipa)
.PHONY: build-ios
build-ios:
	cd $(APP_DIR) && pnpm tauri ios build
	@echo "✅ iOS build done"

# ─── Build All (native platform only) ────────────────────────

.PHONY: build-all
build-all: build-windows build-macos build-linux build-android build-ios
	@echo "✅ All platforms built"

# ─── Release Packaging ───────────────────────────────────────
# Copies built artifacts into releases/vX.Y.Z/

RELEASE_DIR = releases/v$(VERSION)

.PHONY: release-android
release-android:
	@mkdir -p $(RELEASE_DIR)
	@cp $(APP_DIR)/src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk \
		$(RELEASE_DIR)/hazardo-v$(VERSION).apk 2>/dev/null || \
	 cp $(APP_DIR)/src-tauri/gen/android/app/build/outputs/apk/release/app-release.apk \
		$(RELEASE_DIR)/hazardo-v$(VERSION).apk
	@echo "📦 APK → $(RELEASE_DIR)/hazardo-v$(VERSION).apk"

.PHONY: release-windows
release-windows:
	@mkdir -p $(RELEASE_DIR)
	@cp "$(APP_DIR)/src-tauri/target/release/bundle/msi/Hazardo_$(VERSION)_x64_en-US.msi" \
		"$(RELEASE_DIR)/hazardo-v$(VERSION)-windows.msi" 2>/dev/null || true
	@cp "$(APP_DIR)/src-tauri/target/release/bundle/nsis/Hazardo_$(VERSION)_x64-setup.exe" \
		"$(RELEASE_DIR)/hazardo-v$(VERSION)-windows-setup.exe" 2>/dev/null || true
	@echo "📦 Windows → $(RELEASE_DIR)/"

.PHONY: release-macos
release-macos:
	@mkdir -p $(RELEASE_DIR)
	@cp "$(APP_DIR)/src-tauri/target/aarch64-apple-darwin/release/bundle/dmg/Hazardo_$(VERSION)_aarch64.dmg" \
		"$(RELEASE_DIR)/hazardo-v$(VERSION)-macos-arm64.dmg" 2>/dev/null || true
	@echo "📦 macOS → $(RELEASE_DIR)/"

.PHONY: release-linux
release-linux:
	@mkdir -p $(RELEASE_DIR)
	@cp $(APP_DIR)/src-tauri/target/release/bundle/deb/hazardo_$(VERSION)_amd64.deb \
		$(RELEASE_DIR)/hazardo-v$(VERSION)-linux.deb 2>/dev/null || true
	@cp $(APP_DIR)/src-tauri/target/release/bundle/appimage/hazardo_$(VERSION)_amd64.AppImage \
		$(RELEASE_DIR)/hazardo-v$(VERSION)-linux.AppImage 2>/dev/null || true
	@echo "📦 Linux → $(RELEASE_DIR)/"

# ─── Clean ────────────────────────────────────────────────────

.PHONY: clean
clean:
	cd $(APP_DIR)/src-tauri && cargo clean
	@echo "🧹 Cleaned Rust build artifacts"