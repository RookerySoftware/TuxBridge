// The backend's commands and events, typed. Outside the app (a plain browser, for working on
// the UI) a mock backend answers instead, with made-up systems.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type Readiness =
  | { state: "ready"; wslVersion: string }
  | { state: "windowsTooOld"; build: number }
  | { state: "needsWindowsUpdate"; build: number; ubr: number }
  | { state: "virtualizationOff"; virtualMachine: string | null }
  | { state: "wslMissing" }
  | { state: "featuresOff" }
  | { state: "restartNeeded" };

export type Hyprland = "no" | "optional" | "included";

/** A distro's own desktop. */
export interface OwnDesktop {
  id: string;
  name: string;
  note: string | null;
  minutes: string;
  sizeGb: number;
}

/** What a system gets at install: its own desktop, Hyprland, or only a terminal. */
export type DesktopChoice = "own" | "hyprland" | "none";
export type DesktopKind = "own" | "hyprland";

export interface CatalogEntry {
  id: string;
  title: string;
  tagline: string;
  description: string;
  source: { kind: "wsl"; name: string } | { kind: "docker"; image: string } | { kind: "omarchy" };
  desktop: OwnDesktop | null;
  hyprland: Hyprland;
  hyprlandNote: string | null;
  thirdParty: string | null;
  minutes: string;
  sizeGb: number;
  recommended: boolean;
  accent: string;
  website: string;
}

export interface DistroInfo {
  catalogId: string | null;
  user: string | null;
  desktop: string | null;
  hyprland: boolean;
  mainKey: string | null;
  createdByEasywsl: boolean;
  firstStartPending: boolean;
  installedAt: number | null;
  probed: boolean;
}

export interface Distro {
  name: string;
  running: boolean;
  isDefault: boolean;
  wslVersion: number;
  basePath: string;
  info: DistroInfo;
  catalog: CatalogEntry | null;
  busy: boolean;
}

export type StepState = "pending" | "running" | "done" | "failed";

export interface Job {
  id: number;
  kind: string;
  distro: string;
  title: string;
  steps: { title: string; estimate: string | null; state: StepState }[];
  state: StepState;
  error: string | null;
  log: string;
  started: number;
  finished: number | null;
  lastLine: string | null;
}

export interface Session {
  distro: string;
  port: number;
  state: "starting" | "running" | "ended" | "failed";
  message: string | null;
}

export interface Listing {
  distros: Distro[];
  /** False when WSL did not answer. */
  responding: boolean;
}

export interface Settings {
  installDir: string | null;
  fullScreen: boolean;
}

export interface InstallRequest {
  catalogId: string;
  name: string;
  user: string | null;
  password: string | null;
  desktop: DesktopChoice;
  mainKey: string | null;
  allowThirdParty: boolean;
  location: string | null;
  setDefault: boolean;
}

export const inApp = "__TAURI_INTERNALS__" in window;

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!inApp) return mock(command, args) as Promise<T>;
  return invoke<T>(command, args);
}

export const api = {
  systemCheck: () => call<Readiness>("system_check"),
  systemFix: (state: string) => call<Readiness>("system_fix", { state }),
  listDistros: () => call<Listing>("list_distros"),
  restartWsl: () => call<void>("restart_wsl"),
  catalog: () => call<CatalogEntry[]>("get_catalog"),
  suggestName: (catalogId: string) => call<string>("suggest_name", { catalogId }),
  suggestUser: () => call<string>("suggest_user"),
  defaultLocation: (name: string) => call<string>("default_location", { name }),
  checkInstall: (request: InstallRequest) => call<string>("check_install", { request }),
  startInstall: (request: InstallRequest) => call<number>("start_install", { request }),
  addDesktop: (name: string, kind: DesktopKind, mainKey: string, allowThirdParty: boolean) =>
    call<number>("add_desktop", { name, kind, mainKey, allowThirdParty }),
  listJobs: () => call<Job[]>("list_jobs"),
  dismissJob: (id: number) => call<void>("dismiss_job", { id }),
  openTerminal: (name: string) => call<void>("open_terminal", { name }),
  openFiles: (name: string) => call<void>("open_files", { name }),
  reveal: (path: string) => call<void>("reveal", { path }),
  stop: (name: string) => call<void>("stop_distro", { name }),
  setDefault: (name: string) => call<void>("set_default", { name }),
  remove: (name: string) => call<void>("remove_distro", { name }),
  openDesktop: (name: string, kind: DesktopKind) => call<void>("open_desktop", { name, kind }),
  listSessions: () => call<Session[]>("list_sessions"),
  getSettings: () => call<Settings>("get_settings"),
  setSettings: (settings: Settings) => call<void>("set_settings", { settings }),
  defaultInstallDir: () => call<string>("default_install_dir"),
};

/** Subscribes to a backend event; returns the function that unsubscribes. */
export function on<T>(event: "job" | "session" | "distros-changed", handler: (payload: T) => void): () => void {
  if (!inApp) {
    const listener = (e: Event) => handler((e as CustomEvent<T>).detail);
    window.addEventListener(`mock:${event}`, listener);
    return () => window.removeEventListener(`mock:${event}`, listener);
  }
  let unlisten: UnlistenFn | null = null;
  let cancelled = false;
  listen<T>(event, (e) => handler(e.payload)).then((u) => (cancelled ? u() : (unlisten = u)));
  return () => {
    cancelled = true;
    unlisten?.();
  };
}

// ---------------------------------------------------------------------------------------------
// The mock backend, for a plain browser. Its state lives in memory; ?setup=featuresOff and the
// like in the address show the setup screens.

const mockCatalog: CatalogEntry[] = [
  { id: "ubuntu", title: "Ubuntu 26.04 LTS", tagline: "The most popular Linux. A safe first choice.", description: "Ubuntu is the Linux most guides and answers online are written for, so help is easy to find. LTS means it gets updates for years.", source: { kind: "wsl", name: "Ubuntu-26.04" }, desktop: { id: "gnome", name: "GNOME", note: null, minutes: "5 to 10", sizeGb: 1.5 }, hyprland: "optional", hyprlandNote: "Ubuntu's Hyprland is older and a little less smooth than on Arch.", thirdParty: null, minutes: "3 to 5", sizeGb: 2, recommended: false, accent: "#E95420", website: "https://ubuntu.com" },
  { id: "arch", title: "Arch Linux", tagline: "Always the newest software. Best with the Hyprland desktop.", description: "Arch updates continuously, so it gets new versions first.", source: { kind: "wsl", name: "archlinux" }, desktop: null, hyprland: "optional", hyprlandNote: null, thirdParty: null, minutes: "3 to 5", sizeGb: 2.5, recommended: true, accent: "#1793D1", website: "https://archlinux.org" },
  { id: "omarchy", title: "Omarchy", tagline: "A beautiful, ready-made Hyprland desktop by DHH. Beta.", description: "Omarchy is Arch Linux with a polished Hyprland desktop.", source: { kind: "omarchy" }, desktop: null, hyprland: "included", hyprlandNote: "Omarchy is a Hyprland desktop; it is always included.", thirdParty: null, minutes: "10 to 20", sizeGb: 11, recommended: false, accent: "#8FBF8F", website: "https://omarchy.org" },
  { id: "mint", title: "Linux Mint 22.3", tagline: "Familiar and calm. Built on Ubuntu.", description: "Linux Mint is known for feeling comfortable to people coming from Windows.", source: { kind: "docker", image: "linuxmintd/mint22.3-amd64" }, desktop: { id: "cinnamon", name: "Cinnamon", note: null, minutes: "5 to 10", sizeGb: 1 }, hyprland: "optional", hyprlandNote: "Mint has no Hyprland of its own; it comes from a community package source.", thirdParty: "Hyprland and a newer graphics driver come from two community package sources (PPAs) that are not part of Linux Mint.", minutes: "4 to 8", sizeGb: 2.5, recommended: false, accent: "#87CF3E", website: "https://linuxmint.com" },
  { id: "debian", title: "Debian", tagline: "Rock-solid and conservative. The base of Ubuntu.", description: "Debian changes slowly and carefully.", source: { kind: "wsl", name: "Debian" }, desktop: { id: "xfce", name: "Xfce", note: null, minutes: "5 to 10", sizeGb: 1 }, hyprland: "no", hyprlandNote: null, thirdParty: null, minutes: "2 to 4", sizeGb: 1.5, recommended: false, accent: "#A81D33", website: "https://www.debian.org" },
];

const mockState = {
  distros: [
    { name: "arch", running: true, isDefault: true, wslVersion: 2, basePath: "C:\\Users\\me\\AppData\\Local\\EasyWSL\\Distros\\arch", info: { catalogId: "arch", user: "me", desktop: null, hyprland: true, mainKey: "SUPER", createdByEasywsl: true, firstStartPending: false, installedAt: 0, probed: false }, catalog: mockCatalog[1], busy: false },
    { name: "Ubuntu", running: false, isDefault: false, wslVersion: 2, basePath: "C:\\Users\\me\\AppData\\Local\\wsl\\ubuntu", info: { catalogId: null, user: null, desktop: null, hyprland: false, mainKey: null, createdByEasywsl: false, firstStartPending: false, installedAt: null, probed: true }, catalog: null, busy: false },
  ] as Distro[],
  jobs: [] as Job[],
  settings: { installDir: null, fullScreen: true } as Settings,
};

function emit(event: string, detail: unknown) {
  window.dispatchEvent(new CustomEvent(`mock:${event}`, { detail }));
}

function mockJob(request: InstallRequest): number {
  const entry = mockCatalog.find((c) => c.id === request.catalogId)!;
  const steps = [`Downloading ${entry.title}`, `Setting up your account (${request.user})`];
  if (request.desktop === "hyprland") steps.push("Installing the Hyprland desktop");
  if (request.desktop === "own" && entry.desktop) steps.push(`Installing the ${entry.desktop.name} desktop`);
  steps.push("Finishing");
  const job: Job = { id: Date.now(), kind: "install", distro: request.name, title: `Installing ${entry.title}`, steps: steps.map((t) => ({ title: t, estimate: "1 to 2 minutes", state: "pending" })), state: "running", error: null, log: "C:\\mock.log", started: Date.now() / 1000, finished: null, lastLine: null };
  mockState.jobs.push(job);
  let i = 0;
  const tick = () => {
    job.steps.forEach((s, n) => (s.state = n < i ? "done" : n === i ? "running" : "pending"));
    job.lastLine = i < steps.length ? `working on: ${steps[i]}` : null;
    if (i === steps.length) {
      job.state = "done";
      job.finished = Date.now() / 1000;
      mockState.distros.push({ name: request.name, running: false, isDefault: false, wslVersion: 2, basePath: "", info: { catalogId: entry.id, user: request.user, desktop: request.desktop === "own" ? entry.desktop?.id ?? null : null, hyprland: request.desktop === "hyprland", mainKey: request.mainKey, createdByEasywsl: true, firstStartPending: false, installedAt: 0, probed: false }, catalog: entry, busy: false });
      emit("distros-changed", null);
    }
    emit("job", { ...job, steps: job.steps.map((s) => ({ ...s })) });
    if (i++ < steps.length) setTimeout(tick, 1500);
  };
  setTimeout(tick, 300);
  return job.id;
}

async function mock(command: string, args?: Record<string, unknown>): Promise<unknown> {
  await new Promise((r) => setTimeout(r, 150));
  const a = (args ?? {}) as Record<string, any>;
  switch (command) {
    case "system_check": {
      const forced = new URLSearchParams(location.search).get("setup");
      return forced ? { state: forced, build: 19045, ubr: 1000, virtualMachine: null } : { state: "ready", wslVersion: "2.5.9.0" };
    }
    case "system_fix": return { state: "restartNeeded" };
    case "list_distros": return { responding: !new URLSearchParams(location.search).has("stuck"), distros: mockState.distros.map((d) => ({ ...d, busy: mockState.jobs.some((j) => j.distro === d.name && j.state === "running") })) };
    case "get_catalog": return mockCatalog;
    case "suggest_name": return mockState.distros.some((d) => d.name === a.catalogId) ? `${a.catalogId}2` : a.catalogId;
    case "suggest_user": return "me";
    case "default_location": return `C:\\Users\\me\\AppData\\Local\\EasyWSL\\Distros\\${a.name}`;
    case "check_install": return `C:\\Users\\me\\AppData\\Local\\EasyWSL\\Distros\\${a.request.name}`;
    case "start_install": return mockJob(a.request);
    case "list_jobs": return mockState.jobs;
    case "dismiss_job": mockState.jobs = mockState.jobs.filter((j) => j.id !== a.id); return null;
    case "stop_distro": mockState.distros.forEach((d) => d.name === a.name && (d.running = false)); emit("distros-changed", null); return null;
    case "remove_distro": mockState.distros = mockState.distros.filter((d) => d.name !== a.name); emit("distros-changed", null); return null;
    case "open_desktop":
      emit("session", { distro: a.name, port: 5900, state: "starting", message: "Starting the desktop" });
      setTimeout(() => emit("session", { distro: a.name, port: 5900, state: "running", message: null }), 1500);
      return null;
    case "list_sessions": return [];
    case "get_settings": return mockState.settings;
    case "set_settings": mockState.settings = a.settings; return null;
    case "default_install_dir": return "C:\\Users\\me\\AppData\\Local\\EasyWSL\\Distros";
    default: return null;
  }
}
