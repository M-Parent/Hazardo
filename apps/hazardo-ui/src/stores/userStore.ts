import { writable, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import type { User } from "$lib/types";

export const users = writable<User[]>([]);
export const selectedUser = writable<User | null>(null);

export async function loadUsers(): Promise<void> {
  try {
    const result = await invoke<User[]>("get_users");
    users.set(result);
    const current = get(selectedUser);
    if (!current && result.length > 0) {
      selectedUser.set(result[0]);
    }
  } catch (e) {
    console.error("get_users failed", e);
  }
}

export async function createNewUser(userName: string): Promise<User | null> {
  try {
    const created = await invoke<User>("create_user", { userName });
    users.update((u) => [...u, created]);
    selectedUser.set(created);
    return created;
  } catch (e) {
    console.error("create_user failed", e);
    return null;
  }
}

export async function deleteUserById(userId: number): Promise<boolean> {
  try {
    await invoke("delete_user", { userId });
    users.update((u) => u.filter((x) => x.user_id !== userId));
    const current = get(selectedUser);
    if (current?.user_id === userId) {
      const remaining = get(users);
      selectedUser.set(remaining.length > 0 ? remaining[0] : null);
    }
    return true;
  } catch (e) {
    console.error("delete_user failed", e);
    return false;
  }
}
