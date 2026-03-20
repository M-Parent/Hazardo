import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export type Theme = "light" | "dark";

export const currentTheme = writable<Theme>("light");

export function applyTheme(theme: Theme): void {
  if (theme === "dark") {
    document.documentElement.classList.add("dark");
  } else {
    document.documentElement.classList.remove("dark");
  }

  // Update meta theme-color to match chosen theme (overrides prefers-color-scheme)
  const color = theme === "dark" ? "#08050A" : "#F8F6FA";
  document
    .querySelectorAll('meta[name="theme-color"]')
    .forEach((el) => el.remove());
  const meta = document.createElement("meta");
  meta.name = "theme-color";
  meta.content = color;
  document.head.appendChild(meta);

  // Update Android status bar icons via native bridge
  const callNative = () => {
    try {
      const native = (window as any).HazardoNative;
      if (native?.setDarkMode) {
        native.setDarkMode(theme === "dark");
        return true;
      }
    } catch (_) {}
    return false;
  };
  if (!callNative()) {
    // Bridge may not be injected yet — retry with increasing delays
    const delays = [200, 500, 1000, 2000];
    delays.forEach((d) => setTimeout(callNative, d));
  }
}

export async function loadThemeSetting(userId: number): Promise<void> {
  try {
    const settings = await invoke<
      { setting_key: string; setting_value: string }[]
    >("get_all_settings", { userId });
    const map = new Map(settings.map((s) => [s.setting_key, s.setting_value]));
    const theme = map.get("app_theme") as Theme;
    if (theme === "light" || theme === "dark") {
      currentTheme.set(theme);
      applyTheme(theme);
    }
  } catch (_) {}
}

export async function saveThemeSetting(
  userId: number,
  theme: Theme,
): Promise<void> {
  currentTheme.set(theme);
  applyTheme(theme);
  try {
    await invoke("set_setting", { userId, key: "app_theme", value: theme });
  } catch (_) {}
}
