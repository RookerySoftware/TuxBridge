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
  /** Its docker uses TuxBridge's engine. */
  docker: boolean;
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
  /** null: fit the screen or window; "screen": the screen's size; "WxH": a fixed size. */
  resolution: string | null;
  frameRate: number;
  /** Start Docker when the app opens. */
  startDocker: boolean;
}

export interface DockerStatus {
  /** TuxBridge's engine is set up. */
  installed: boolean;
  running: boolean;
  version: string | null;
  desktopInstalled: boolean;
  desktopRunning: boolean;
}

export type DockerEngine = "easywsl" | "desktop";

export interface Container {
  engine: DockerEngine;
  id: string;
  name: string;
  image: string;
  state: string;
  status: string;
  /** Published TCP ports on the PC. */
  ports: number[];
  created: string;
  /** The gallery app it was installed as. */
  app: string | null;
}

export interface DiskUse {
  kind: string;
  count: string;
  size: string;
  reclaimable: string;
}

export interface AppQuestion {
  id: string;
  label: string;
  hint: string | null;
  kind: "password" | "folder";
  required: boolean;
  default: string | null;
}

export interface GalleryApp {
  id: string;
  name: string;
  category: string;
  tagline: string;
  description: string;
  image: string;
  ports: { host: number; container: number }[];
  web: boolean;
  questions: AppQuestion[];
  note: string | null;
  after: string;
  download: string;
  accent: string;
  website: string;
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

export interface AiModel {
  id: "small" | "standard" | "large";
  tag: string;
  name: string;
  description: string;
  sizeGb: number;
}

export interface AiStatus {
  installed: boolean;
  running: boolean;
  /** The installed model's id. */
  model: AiModel["id"] | null;
  models: AiModel[];
  recommended: AiModel["id"];
  runtimeGb: number;
  /** What was found: the graphics card and memory. */
  hardware: string;
}

export interface AiMessage {
  role: "system" | "user" | "assistant";
  content: string;
}

/** A piece of the assistant's answer, as it is written. */
export interface AiChunk {
  id: number;
  text: string;
  done: boolean;
  error: string | null;
}

/** An AI app the TuxBridge skill can be added to. */
export interface SkillTarget {
  id: "claude" | "codex";
  name: string;
  detected: boolean;
  installed: boolean;
  /** The skill's folder. */
  path: string;
}

export const inApp = "__TAURI_INTERNALS__" in window;

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!inApp) return mock(command, args) as Promise<T>;
  return invoke<T>(command, args);
}

const RECENT_KEY = "easywsl.recent";

/** The systems opened most recently (terminal, desktop or files), newest first. Kept in this PC's
 *  localStorage, which the tray menu's window shares. */
export function recentSystems(): string[] {
  try {
    return JSON.parse(localStorage.getItem(RECENT_KEY) ?? "[]") as string[];
  } catch {
    return [];
  }
}

function used(name: string) {
  try {
    localStorage.setItem(RECENT_KEY, JSON.stringify([name, ...recentSystems().filter((n) => n !== name)].slice(0, 10)));
  } catch {
    /* private mode, ignore */
  }
}

/** The desktops a system has, the one opened by default first. */
export function desktopsOf(distro: Distro): { kind: DesktopKind; name: string }[] {
  const list: { kind: DesktopKind; name: string }[] = [];
  if (distro.info.desktop) {
    const name = distro.catalog?.desktop?.name ?? distro.info.desktop.charAt(0).toUpperCase() + distro.info.desktop.slice(1);
    list.push({ kind: "own", name });
  }
  if (distro.info.hyprland) list.push({ kind: "hyprland", name: "Hyprland" });
  return list;
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
  jobLogTail: (id: number) => call<string>("job_log_tail", { id }),
  openTerminal: (name: string) => (used(name), call<void>("open_terminal", { name })),
  /** Sets up a Learn quest in a system and opens a terminal there. */
  startQuest: (name: string, quest: string) => call<void>("start_quest", { name, quest }),
  openFiles: (name: string) => (used(name), call<void>("open_files", { name })),
  reveal: (path: string) => call<void>("reveal", { path }),
  stop: (name: string) => call<void>("stop_distro", { name }),
  setDefault: (name: string) => call<void>("set_default", { name }),
  remove: (name: string) => call<void>("remove_distro", { name }),
  openDesktop: (name: string, kind: DesktopKind) => (used(name), call<void>("open_desktop", { name, kind })),
  /** The tray menu: fit its window to `height` and show it; hide it. */
  trayFit: (height: number) => call<void>("tray_fit", { height }),
  trayHide: () => call<void>("tray_hide"),
  /** Shows the main window, on a page if given. */
  showWindow: (page?: string) => call<void>("show_window", { page }),
  quit: () => call<void>("quit_app"),
  listSessions: () => call<Session[]>("list_sessions"),
  getSettings: () => call<Settings>("get_settings"),
  setSettings: (settings: Settings) => call<void>("set_settings", { settings }),
  defaultInstallDir: () => call<string>("default_install_dir"),
  dockerStatus: () => call<DockerStatus>("docker_status"),
  dockerEnable: () => call<number>("docker_enable"),
  dockerStart: () => call<void>("docker_start"),
  dockerStop: () => call<void>("docker_stop"),
  dockerRemove: () => call<void>("docker_remove"),
  containers: () => call<Container[]>("docker_containers"),
  containerAction: (engine: DockerEngine, id: string, action: "start" | "stop" | "restart" | "remove") =>
    call<void>("docker_container_action", { engine, id, action }),
  /** Updates a container to the newest version of its image; returns the job. */
  updateContainer: (engine: DockerEngine, id: string, name: string) => call<number>("docker_update", { engine, id, name }),
  containerLogs: (engine: DockerEngine, id: string) => call<string>("docker_logs", { engine, id }),
  dockerDisk: () => call<DiskUse[]>("docker_disk"),
  dockerCleanUp: (images: boolean) => call<string>("docker_clean_up", { images }),
  apps: () => call<GalleryApp[]>("list_apps"),
  installApp: (id: string, answers: Record<string, string>) => call<number>("install_app", { id, answers }),
  /** Updates a system's programs: the job, or null when its own tool opened in a terminal (Omarchy). */
  updateSystem: (name: string) => call<number | null>("update_system", { name }),
  dockerUseIn: (name: string) => call<number>("docker_use_in", { name }),
  aiStatus: () => call<AiStatus>("ai_status"),
  /** Installs the assistant with this model, or changes its model; returns the job. */
  aiInstall: (model: AiModel["id"]) => call<number>("ai_install", { model }),
  aiRemove: () => call<void>("ai_remove"),
  /** Starts an answer; it arrives as "ai-chunk" events with this id. */
  aiChat: (id: number, messages: AiMessage[]) => call<void>("ai_chat", { id, messages }),
  aiCancel: (id: number) => call<void>("ai_cancel", { id }),
  /** Loads the model ahead of the first question. */
  aiWarmUp: () => call<void>("ai_warm_up"),
  skillsStatus: () => call<SkillTarget[]>("skills_status"),
  skillInstall: (target: SkillTarget["id"]) => call<void>("skill_install", { target }),
  skillRemove: (target: SkillTarget["id"]) => call<void>("skill_remove", { target }),
  /** Saves the skill as a .zip in Downloads; returns its path. */
  skillExport: () => call<string>("skill_export"),
};

/** Subscribes to a backend event; returns the function that unsubscribes. */
export function on<T>(event: "job" | "session" | "distros-changed" | "docker-changed" | "ai-changed" | "ai-chunk" | "navigate" | "tray-shown", handler: (payload: T) => void): () => void {
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
  { id: "kali", title: "Kali Linux", tagline: "For security testing and learning about it.", description: "Kali comes with the tools security professionals use to test systems.", source: { kind: "wsl", name: "kali-linux" }, desktop: { id: "xfce", name: "Xfce", note: null, minutes: "5 to 10", sizeGb: 1 }, hyprland: "optional", hyprlandNote: null, thirdParty: null, minutes: "3 to 5", sizeGb: 2, recommended: false, accent: "#367BF0", website: "https://www.kali.org" },
  { id: "parrot", title: "Parrot Security", tagline: "Security testing, privacy and development tools. A friendly Kali alternative.", description: "Parrot is built on Debian like Kali, with security, forensics, privacy and programming tools.", source: { kind: "docker", image: "parrotsec/core" }, desktop: { id: "kde", name: "KDE Plasma", note: null, minutes: "10 to 15", sizeGb: 2.5 }, hyprland: "no", hyprlandNote: null, thirdParty: null, minutes: "3 to 5", sizeGb: 1, recommended: false, accent: "#05EEFF", website: "https://www.parrotsec.org" },
];

const mockState = {
  distros: [
    { name: "arch", running: true, isDefault: true, wslVersion: 2, basePath: "C:\\Users\\me\\AppData\\Local\\TuxBridge\\Distros\\arch", info: { catalogId: "arch", user: "me", desktop: null, hyprland: true, mainKey: "SUPER", createdByEasywsl: true, firstStartPending: false, installedAt: 0, probed: false, docker: false }, catalog: mockCatalog[1], busy: false },
    { name: "Ubuntu", running: false, isDefault: false, wslVersion: 2, basePath: "C:\\Users\\me\\AppData\\Local\\wsl\\ubuntu", info: { catalogId: null, user: null, desktop: null, hyprland: false, mainKey: null, createdByEasywsl: false, firstStartPending: false, installedAt: null, probed: true, docker: false }, catalog: null, busy: false },
  ] as Distro[],
  jobs: [] as Job[],
  settings: { installDir: null, fullScreen: true, resolution: null, frameRate: 60, startDocker: true } as Settings,
  docker: { installed: !new URLSearchParams(location.search).has("nodocker"), running: true, version: "29.9.2", desktopInstalled: false, desktopRunning: false } as DockerStatus,
};

/** For trying prompts in the browser: the assistant's server (ai.rs) answers directly. */
async function realAi(id: number, messages: AiMessage[]) {
  const model = new URLSearchParams(location.search).get("realai") || "qwen3.5:4b-q4_K_M";
  try {
    const response = await fetch("http://127.0.0.1:11435/api/chat", {
      method: "POST",
      body: JSON.stringify({ model, messages, stream: true, think: false, options: { num_ctx: 16384, temperature: 0.3 } }),
    });
    const reader = response.body!.pipeThrough(new TextDecoderStream()).getReader();
    let pending = "";
    for (;;) {
      const { value, done } = await reader.read();
      if (done) break;
      pending += value;
      const lines = pending.split("\n");
      pending = lines.pop()!;
      for (const line of lines.filter(Boolean)) emit("ai-chunk", { id, text: JSON.parse(line).message?.content ?? "", done: false, error: null });
    }
    emit("ai-chunk", { id, text: "", done: true, error: null });
  } catch (e) {
    emit("ai-chunk", { id, text: "", done: true, error: String(e) });
  }
}

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
      mockState.distros.push({ name: request.name, running: false, isDefault: false, wslVersion: 2, basePath: "", info: { catalogId: entry.id, user: request.user, desktop: request.desktop === "own" ? entry.desktop?.id ?? null : null, hyprland: request.desktop === "hyprland", mainKey: request.mainKey, createdByEasywsl: true, firstStartPending: false, installedAt: 0, probed: false, docker: false }, catalog: entry, busy: false });
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
    case "default_location": return `C:\\Users\\me\\AppData\\Local\\TuxBridge\\Distros\\${a.name}`;
    case "check_install": return `C:\\Users\\me\\AppData\\Local\\TuxBridge\\Distros\\${a.request.name}`;
    case "start_install": return mockJob(a.request);
    case "list_jobs": return mockState.jobs;
    case "dismiss_job": mockState.jobs = mockState.jobs.filter((j) => j.id !== a.id); return null;
    case "stop_distro": mockState.distros.forEach((d) => d.name === a.name && (d.running = false)); emit("distros-changed", null); return null;
    case "remove_distro": mockState.distros = mockState.distros.filter((d) => d.name !== a.name); emit("distros-changed", null); return null;
    case "open_desktop":
      emit("session", { distro: a.name, state: "starting", message: "Starting the desktop" });
      setTimeout(() => emit("session", { distro: a.name, state: "running", message: null }), 1500);
      return null;
    case "start_quest": return null;
    case "list_sessions": return [];
    case "get_settings": return mockState.settings;
    case "set_settings": mockState.settings = a.settings; return null;
    case "default_install_dir": return "C:\\Users\\me\\AppData\\Local\\TuxBridge\\Distros";
    case "docker_status": return mockState.docker;
    case "docker_enable": mockState.docker = { ...mockState.docker, installed: true }; emit("docker-changed", null); return 0;
    case "docker_start": mockState.docker = { ...mockState.docker, running: true }; return null;
    case "docker_stop": mockState.docker = { ...mockState.docker, running: false }; return null;
    case "docker_containers": return mockState.docker.running ? [
      { engine: "easywsl", id: "a1", name: "jellyfin", image: "jellyfin/jellyfin:latest", state: "running", status: "Up 2 hours", ports: [8096], created: "", app: "jellyfin" },
      { engine: "easywsl", id: "b2", name: "postgres", image: "postgres:17", state: "exited", status: "Exited (0) 3 days ago", ports: [], created: "", app: "postgres" },
    ] : [];
    case "ai_status": return {
      installed: !new URLSearchParams(location.search).has("noai"), running: false, model: "standard", recommended: "standard", runtimeGb: 1.6,
      hardware: "NVIDIA GeForce RTX 4060 with 8 GB, 32 GB of memory",
      models: [
        { id: "small", tag: "qwen3.5:2b-q4_K_M", name: "Small", description: "For PCs with little memory. Quick, but makes more mistakes.", sizeGb: 1.9 },
        { id: "standard", tag: "qwen3.5:4b-q4_K_M", name: "Standard", description: "Good answers on most PCs.", sizeGb: 3.3 },
        { id: "large", tag: "qwen3.5:9b-q4_K_M", name: "Large", description: "The best answers. Needs a graphics card with 12 GB or more.", sizeGb: 6.6 },
      ],
    };
    case "ai_chat": {
      const id = args?.id as number;
      if (new URLSearchParams(location.search).has("realai")) {
        realAi(id, args?.messages as AiMessage[]);
        return null;
      }
      const words = "In **Ubuntu**, open a terminal and run:\n\n```\nsudo apt update && sudo apt upgrade\n```\n\nOr use **Update** in the system's menu. See [Installing software](software.md).".split(/(?<= )/);
      words.forEach((w, i) => setTimeout(() => emit("ai-chunk", { id, text: w, done: false, error: null }), 40 * i));
      setTimeout(() => emit("ai-chunk", { id, text: "", done: true, error: null }), 40 * words.length);
      return null;
    }
    case "skills_status": return [
      { id: "claude", name: "Claude Code", detected: true, installed: false, path: "C:\\Users\\me\\.claude\\skills\\tuxbridge" },
      { id: "codex", name: "Codex", detected: false, installed: false, path: "C:\\Users\\me\\.agents\\skills\\tuxbridge" },
    ];
    case "skill_export": return "C:\\Users\\me\\Downloads\\tuxbridge-skill.zip";
    case "docker_update": return 0;
    case "update_system": return 0;
    case "docker_logs": return "Starting...\nReady.";
    case "docker_disk": return [{ kind: "Images", count: "4", size: "1.2GB", reclaimable: "300MB (25%)" }, { kind: "Containers", count: "2", size: "12MB", reclaimable: "0B" }, { kind: "Local Volumes", count: "3", size: "800MB", reclaimable: "0B" }, { kind: "Build Cache", count: "0", size: "0B", reclaimable: "0B" }];
    case "docker_clean_up": return "Total reclaimed space: 300MB";
    case "list_apps": return [
      { id: "jellyfin", name: "Jellyfin", category: "Media", tagline: "Your own Netflix for the films, shows and music on your PC.", description: "Jellyfin organises a folder of videos and music.", image: "jellyfin/jellyfin:latest", ports: [{ host: 8096, container: 8096 }], web: true, questions: [{ id: "media", label: "Your videos and music", hint: "A Windows folder.", kind: "folder", required: true, default: "%USERPROFILE%\\Videos" }], note: null, after: "Open it and follow the set-up.", download: "about a minute", accent: "#AA5CC3", website: "https://jellyfin.org" },
      { id: "postgres", name: "PostgreSQL", category: "Databases", tagline: "The popular open-source database.", description: "PostgreSQL for development.", image: "postgres:17", ports: [{ host: 5432, container: 5432 }], web: false, questions: [{ id: "password", label: "Password", hint: "For the user postgres.", kind: "password", required: true, default: null }], note: null, after: "Connect to localhost.", download: "under a minute", accent: "#336791", website: "https://www.postgresql.org" },
    ];
    default: return null;
  }
}
