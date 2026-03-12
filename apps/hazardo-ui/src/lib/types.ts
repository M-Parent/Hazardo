export interface User {
  user_id: number;
  user_name: string;
}

export interface Category {
  category_id: number;
  user_id: number;
  category_name: string;
  category_icon: string;
  is_default: number;
}

export interface Item {
  item_id: number;
  category_id: number;
  user_id: number;
  item_name: string;
  time_pref: string;
  vibe_pref: string;
  location: string | null;
  description: string | null;
  image_path: string | null;
  notes: string | null;
  is_picked: number;
}

export interface Pick {
  pick_id: number;
  user_id: number;
  item_id: number;
  category_id: number;
  item_name: string;
  category_name: string;
  category_icon: string;
  pick_date: string;
  time_pref: string;
  vibe_pref: string;
  ai_recommendation: string | null;
  notes: string | null;
  image_path: string | null;
  location: string | null;
  created_at: string;
}

export interface AppSetting {
  setting_key: string;
  setting_value: string;
}

export interface ChatMessage {
  message_id: number;
  user_id: number;
  role: "user" | "assistant";
  content: string;
  created_at: string;
}

export interface Device {
  device_id: number;
  device_name: string;
}

export type TimePref = "AM" | "PM" | "Night" | "Mixed";
export type VibePref = "Friend" | "Date" | "Family" | "Mixed";

export type IconName =
  | "home"
  | "vault"
  | "calendar"
  | "chatbot"
  | "setting"
  | "picked"
  | "chevron-down"
  | "chevron-right"
  | "plus"
  | "search"
  | "trash"
  | "arrow-up"
  | "close"
  | "microphone"
  | "send"
  | "edit"
  | "check"
  | "image"
  | "clock"
  | "users"
  | "wifi-off"
  | "dice"
  | "activity"
  | "restaurant"
  | "board-game"
  | "valentine"
  | "movie"
  | "gift"
  | "workout"
  | "travel"
  | "cooking"
  | "misc"
  | "recycle"
  | "import"
  | "export"
  | "palette"
  | "globe"
  | "monitor"
  | "sync"
  | "brain"
  | "file-text"
  | "info"
  | "thermometer"
  | "navigation"
  | "map-pin";
