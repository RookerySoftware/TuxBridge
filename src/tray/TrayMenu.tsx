// The tray menu: the small window that opens when you right-click Pip in the notification area.
// Recent systems (left click opens the desktop, or the terminal when there's none; right click
// offers more), then Open TuxBridge, Settings and Quit. It fits its window to what it shows.

import { useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";
import { LayoutDashboard, Monitor, Power, Settings as SettingsIcon, SquareTerminal, X } from "lucide-react";
import { api, desktopsOf, on, recentSystems, type Distro } from "../api";
import Penguin from "../companion/Penguin";

/** How many systems the menu lists. */
const SHOWN = 5;

/** Recently used first, then the running ones, then the rest. */
function ordered(distros: Distro[]): Distro[] {
  const recent = recentSystems();
  const rank = (d: Distro) => {
    const i = recent.indexOf(d.name);
    return i >= 0 ? i : 100 + (d.running ? 0 : 50);
  };
  return [...distros].sort((a, b) => rank(a) - rank(b) || a.name.localeCompare(b.name)).slice(0, SHOWN);
}

export default function TrayMenu() {
  const [distros, setDistros] = useState<Distro[] | null>(null);
  const [menuFor, setMenuFor] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const panel = useRef<HTMLDivElement>(null);

  const load = useCallback(() => {
    setMenuFor(null);
    setError(null);
    api.listDistros().then((l) => setDistros(ordered(l.distros))).catch(() => setDistros([]));
  }, []);

  useEffect(() => {
    load();
    return on("tray-shown", load);
  }, [load]);

  // Fit the window to the panel whenever what it shows changes (this also shows it).
  useLayoutEffect(() => {
    if (distros && panel.current) api.trayFit(Math.ceil(panel.current.getBoundingClientRect().height) + 2).catch(() => {});
  }, [distros, menuFor, error]);

  useEffect(() => {
    const key = (e: KeyboardEvent) => e.key === "Escape" && api.trayHide();
    const noMenu = (e: MouseEvent) => e.preventDefault();
    window.addEventListener("keydown", key);
    window.addEventListener("contextmenu", noMenu);
    return () => {
      window.removeEventListener("keydown", key);
      window.removeEventListener("contextmenu", noMenu);
    };
  }, []);

  const run = (action: () => Promise<unknown>) => {
    action()
      .then(() => api.trayHide())
      .catch((e) => setError(String(e)));
  };

  const open = (d: Distro) => {
    const desktop = desktopsOf(d)[0];
    run(() => (desktop ? api.openDesktop(d.name, desktop.kind) : api.openTerminal(d.name)));
  };

  return (
    <div ref={panel} className="m-px overflow-hidden rounded-xl border border-line bg-surface text-sm text-ink shadow-xl">
      <div className="flex items-center gap-2 border-b border-line px-3 py-2.5">
        <Penguin size={26} />
        <span className="flex-1 font-display font-semibold">TuxBridge</span>
        <button aria-label="Close" className="rounded p-1 text-muted hover:bg-surface-2 hover:text-ink" onClick={() => api.trayHide()}>
          <X className="size-3.5" />
        </button>
      </div>

      <div className="px-1.5 py-1.5">
        <p className="px-2 pb-1 pt-0.5 text-[11px] font-semibold uppercase tracking-wide text-muted">Recent systems</p>
        {distros?.length === 0 && <p className="px-2 py-1.5 text-muted">No Linux systems yet.</p>}
        {distros?.map((d) => {
          const desktops = desktopsOf(d);
          return (
            <div key={d.name}>
              <button
                className={`flex w-full items-center gap-2.5 rounded-lg px-2 py-1.5 text-left transition hover:bg-surface-2 ${menuFor === d.name ? "bg-surface-2" : ""}`}
                title={desktops.length ? `Click: open the ${desktops[0].name} desktop. Right-click: more.` : "Click: open a terminal. Right-click: more."}
                onClick={() => open(d)}
                onContextMenu={(e) => {
                  e.preventDefault();
                  setMenuFor(menuFor === d.name ? null : d.name);
                }}
              >
                <span
                  className="flex size-7 shrink-0 items-center justify-center rounded-md font-display text-xs font-bold text-white"
                  style={{ background: d.catalog?.accent ?? "#5d6676" }}
                >
                  {(d.catalog?.title ?? d.name).charAt(0).toUpperCase()}
                </span>
                <span className="min-w-0 flex-1">
                  <span className="block truncate font-medium">{d.name}</span>
                  <span className="block truncate text-xs text-muted">
                    {d.running ? "Running" : "Stopped"}
                    {desktops.length ? ` · ${desktops.map((x) => x.name).join(", ")}` : ""}
                  </span>
                </span>
                {d.running && <span className="size-2 shrink-0 rounded-full bg-good" />}
              </button>
              {menuFor === d.name && (
                <div className="mb-1 ml-9 mt-0.5 space-y-0.5 border-l border-line pl-2 animate-in">
                  {desktops.map((x) => (
                    <Item key={x.kind} icon={x.kind === "hyprland" ? <LayoutDashboard className="size-4" /> : <Monitor className="size-4" />} onClick={() => run(() => api.openDesktop(d.name, x.kind))}>
                      Open {desktops.length > 1 ? `the ${x.name} desktop` : "desktop"}
                    </Item>
                  ))}
                  <Item icon={<SquareTerminal className="size-4" />} onClick={() => run(() => api.openTerminal(d.name))}>
                    Open terminal
                  </Item>
                  {d.running && (
                    <Item icon={<Power className="size-4" />} onClick={() => run(() => api.stop(d.name))}>
                      Stop
                    </Item>
                  )}
                </div>
              )}
            </div>
          );
        })}
        {error && <p className="px-2 py-1.5 text-xs text-bad">{error}</p>}
      </div>

      <div className="border-t border-line px-1.5 py-1.5">
        <Item icon={<Penguin size={16} />} onClick={() => api.showWindow()}>
          Open TuxBridge
        </Item>
        <Item icon={<SettingsIcon className="size-4" />} onClick={() => api.showWindow("settings")}>
          Settings
        </Item>
        <Item icon={<Power className="size-4" />} onClick={() => api.quit()}>
          Quit TuxBridge
        </Item>
      </div>
    </div>
  );
}

function Item({ icon, onClick, children }: { icon: React.ReactNode; onClick: () => void; children: React.ReactNode }) {
  return (
    <button className="flex w-full items-center gap-2.5 rounded-lg px-2 py-1.5 text-left transition hover:bg-surface-2" onClick={onClick}>
      <span className="flex w-4 justify-center text-muted">{icon}</span>
      {children}
    </button>
  );
}
