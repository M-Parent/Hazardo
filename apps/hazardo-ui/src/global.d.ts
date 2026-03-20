interface HazardoNative {
  setDarkMode(isDark: boolean): void;
}

interface Window {
  HazardoNative?: HazardoNative;
}
