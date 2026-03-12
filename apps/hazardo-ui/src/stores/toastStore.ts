import { writable } from "svelte/store";

export interface Toast {
  id: number;
  message: string;
  type: "success" | "error";
}

let nextId = 0;
export const toasts = writable<Toast[]>([]);

export function showToast(
  message: string,
  type: "success" | "error" = "success",
  duration = 3000,
) {
  const id = ++nextId;
  toasts.update((t) => [...t, { id, message, type }]);
  setTimeout(() => {
    toasts.update((t) => t.filter((x) => x.id !== id));
  }, duration);
}
