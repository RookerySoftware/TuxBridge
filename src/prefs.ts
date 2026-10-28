// Preferences kept in this window (localStorage) that several pages share: Kids mode and the
// companion. Any component reading one re-renders when another changes it.

import { useSyncExternalStore } from "react";

const listeners = new Set<() => void>();

function read<T>(key: string, fallback: T): T {
  try {
    const raw = localStorage.getItem(key);
    return raw ? (JSON.parse(raw) as T) : fallback;
  } catch {
    return fallback;
  }
}

function write(key: string, value: unknown) {
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch {
    /* private mode, ignore */
  }
  for (const l of listeners) l();
}

function subscribe(l: () => void) {
  listeners.add(l);
  return () => listeners.delete(l);
}

function usePref<T extends boolean | string>(key: string, fallback: T): [T, (v: T) => void] {
  const value = useSyncExternalStore(subscribe, () => read(key, fallback));
  return [value, (v: T) => write(key, v)];
}

/** Kids mode: gentler content on Learn, and the kids' companion. (The key predates this file.) */
export const useKidsMode = () => usePref<boolean>("easywsl.learn.kids", false);

/** The companion for everyone else: off until turned on in Settings or by the penguin itself. */
export const useCompanionOn = () => usePref<boolean>("easywsl.companion", false);
