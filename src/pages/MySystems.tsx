// The systems on this PC: what they are, whether they run, and the things to do with them.

import { useEffect, useRef, useState } from "react";
import {
  AlertTriangle, BookOpen, ChevronDown, Download, FolderOpen, Loader2, LayoutDashboard, Monitor, Plus, Power, Sparkles, Star, SquareTerminal, Trash2,
} from "lucide-react";
import { api, desktopsOf, type DesktopKind, type Distro, type Job, type Session } from "../api";
import { Badge, Button, Card, Dialog, Menu, Monogram, Notice, Toggle, Field, inputClass } from "../components/ui";
import JobCard from "../components/JobCard";
import { guides } from "../guides";

function RemoveDialog({ distro, onClose }: { distro: Distro; onClose: () => void }) {
  const [typed, setTyped] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const foreign = !distro.info.createdByEasywsl;
  return (
    <Dialog title={`Remove ${distro.name}?`} onClose={onClose}>
      <div className="space-y-4">
        <p className="text-muted">
          This deletes the system and <b className="text-ink">everything in it</b>: its files, its programs, its settings. It cannot be
          undone. Your Windows files are not touched.
        </p>
        {foreign && (
          <Notice tone="warn" icon={<AlertTriangle className="size-4 text-warn" />}>
            Pengi did not install this system, so it may hold work of yours. Copy anything you need first (<i>Open files</i>).
          </Notice>
        )}
        <Field label={`Type ${distro.name} to confirm`}>
          <input className={inputClass} value={typed} onChange={(e) => setTyped(e.target.value)} spellCheck={false} />
        </Field>
        {error && <Notice tone="bad">{error}</Notice>}
        <div className="flex justify-end gap-2">
          <Button onClick={onClose}>Keep it</Button>
          <Button
            variant="danger"
            busy={busy}
            disabled={typed !== distro.name}
            onClick={async () => {
              setBusy(true);
              try {
                await api.remove(distro.name);
                onClose();
              } catch (e) {
                setError(String(e));
                setBusy(false);
              }
            }}
          >
            Remove
          </Button>
        </div>
      </div>
    </Dialog>
  );
}

function AddDesktopDialog({ distro, kind, onClose }: { distro: Distro; kind: DesktopKind; onClose: () => void }) {
  const [windowsKey, setWindowsKey] = useState(true);
  const [thirdParty, setThirdParty] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const hyprland = kind === "hyprland";
  const own = distro.catalog?.desktop;
  const needsThirdParty = hyprland ? distro.catalog?.thirdParty : null;
  const name = hyprland ? "Hyprland" : (own?.name ?? "its own");
  return (
    <Dialog title={`Add the ${name} desktop`} onClose={onClose}>
      <div className="space-y-4">
        <p className="text-muted">
          {hyprland
            ? "Hyprland is a modern Linux desktop where windows arrange themselves in tiles. Installing it takes 2 to 5 minutes."
            : `${name} is ${distro.catalog?.title ?? "this system"}'s own desktop, with a menu, a taskbar and windows. Installing it takes ${own?.minutes ?? "5 to 10"} minutes.`}{" "}
          It opens in its own window on Windows. Your files and apps stay as they are.
        </p>
        {hyprland && distro.catalog?.hyprlandNote && <Notice>{distro.catalog.hyprlandNote}</Notice>}
        {!hyprland && own?.note && <Notice>{own.note}</Notice>}
        {hyprland && (
          <Toggle
            checked={windowsKey}
            onChange={setWindowsKey}
            label="Use the Windows key for the desktop's shortcuts"
            description="Recommended. Off: the Alt key instead."
          />
        )}
        {needsThirdParty && (
          <Toggle checked={thirdParty} onChange={setThirdParty} label="Allow the community package sources" description={needsThirdParty} />
        )}
        {error && <Notice tone="bad">{error}</Notice>}
        <div className="flex justify-end gap-2">
          <Button onClick={onClose}>Cancel</Button>
          <Button
            variant="primary"
            busy={busy}
            disabled={!!needsThirdParty && !thirdParty}
            onClick={async () => {
              setBusy(true);
              try {
                await api.addDesktop(distro.name, kind, windowsKey ? "SUPER" : "ALT", thirdParty);
                onClose();
              } catch (e) {
                setError(String(e));
                setBusy(false);
              }
            }}
          >
            Add {name}
          </Button>
        </div>
      </div>
    </Dialog>
  );
}

/** "Open" with the system's default (its desktop, else a terminal), and an arrow for the others. */
function OpenButton({ items, disabled }: { items: { label: string; icon: React.ReactNode; onClick: () => void }[]; disabled?: boolean }) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!open) return;
    const close = (e: MouseEvent) => !ref.current?.contains(e.target as Node) && setOpen(false);
    window.addEventListener("mousedown", close);
    return () => window.removeEventListener("mousedown", close);
  }, [open]);
  const [first] = items;
  const split = items.length > 1;
  return (
    <div className="relative inline-flex" ref={ref}>
      <Button variant="primary" className={split ? "rounded-r-none" : ""} icon={first.icon} disabled={disabled} onClick={first.onClick}>
        {first.label}
      </Button>
      {split && (
        <button
          aria-label="Other ways to open"
          aria-expanded={open}
          disabled={disabled}
          onClick={() => setOpen(!open)}
          className="inline-flex h-9 items-center rounded-r-lg border-l border-black/15 bg-accent px-2 text-accent-ink transition hover:brightness-110
            focus-visible:outline-2 focus-visible:outline-accent disabled:pointer-events-none disabled:opacity-50"
        >
          <ChevronDown className="size-4" />
        </button>
      )}
      {open && (
        <div className="absolute left-0 top-10 z-20 w-60 rounded-xl border border-line bg-surface p-1 shadow-xl animate-in">
          {items.map((i) => (
            <button
              key={i.label}
              onClick={() => {
                setOpen(false);
                i.onClick();
              }}
              className="flex w-full items-center gap-2.5 rounded-lg px-2.5 py-2 text-left text-sm hover:bg-surface-2"
            >
              {i.icon}
              {i.label}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

function DistroCard({ distro, session, onSessionDismiss, onHelp }: { distro: Distro; session?: Session; onSessionDismiss: () => void; onHelp: (topic: string) => void }) {
  const [dialog, setDialog] = useState<"remove" | DesktopKind | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const title = distro.catalog?.title ?? distro.name;
  const accent = distro.catalog?.accent ?? "#6b7280";
  const desktops = desktopsOf(distro);
  const canAddHyprland = !distro.info.hyprland && distro.catalog?.hyprland === "optional";
  const canAddOwn = !distro.info.desktop && !!distro.catalog?.desktop;
  const desktopBusy = session?.state === "starting" || session?.state === "running";

  const run = (action: () => Promise<void>) => () => {
    setError(null);
    setNotice(null);
    action().catch((e) => setError(String(e)));
  };
  const update = run(async () => {
    const job = await api.updateSystem(distro.name);
    if (job === null) setNotice("Omarchy updates with its own tool, which opened in a terminal window. Follow it there.");
  });

  const terminal = { label: "Terminal", icon: <SquareTerminal className="size-4" />, onClick: run(() => api.openTerminal(distro.name)) };
  const openItems = [
    ...desktops.map((d) => ({
      label:
        desktops.length > 1 ? `${d.name} desktop` : distro.info.firstStartPending && d.kind === "hyprland" ? "Set up and open" : "Open desktop",
      icon: d.kind === "hyprland" ? <LayoutDashboard className="size-4" /> : <Monitor className="size-4" />,
      onClick: run(() => api.openDesktop(distro.name, d.kind)),
    })),
    terminal,
  ];

  return (
    <Card className="p-5 animate-in" tour={`distro:${distro.name}`}>
      <div className="flex items-start gap-4">
        <Monogram title={title} accent={accent} />
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-2">
            <h3 className="font-display text-base font-semibold">{distro.name}</h3>
            {distro.isDefault && (
              <span title="Opens when you type wsl in a terminal">
                <Star className="size-3.5 fill-warn text-warn" />
              </span>
            )}
          </div>
          <div className="text-sm text-muted">{distro.catalog ? title : "Linux system"}</div>
          <div className="mt-2 flex flex-wrap gap-1.5">
            {distro.busy ? (
              <Badge tone="accent">
                <Loader2 className="size-3 animate-spin" /> Changing
              </Badge>
            ) : session?.state === "running" ? (
              <Badge tone="good">● Desktop open</Badge>
            ) : distro.running ? (
              <Badge tone="good">● Running</Badge>
            ) : (
              <Badge>Stopped</Badge>
            )}
            {desktops.map((d) => (
              <Badge key={d.kind} tone="accent">
                {d.name} desktop
              </Badge>
            ))}
            {distro.wslVersion === 1 && <Badge tone="warn">WSL 1</Badge>}
          </div>
        </div>
        <Menu
          items={[
            { label: "Update", icon: <Download className="size-4" />, onClick: update, hidden: distro.busy || distro.info.firstStartPending },
            { label: "Stop", icon: <Power className="size-4" />, onClick: run(() => api.stop(distro.name)), hidden: !distro.running },
            { label: "Make default", icon: <Star className="size-4" />, onClick: run(() => api.setDefault(distro.name)), hidden: distro.isDefault },
            {
              label: `Add the ${distro.catalog?.desktop?.name ?? "own"} desktop`,
              icon: <Monitor className="size-4" />,
              onClick: () => setDialog("own"),
              hidden: !canAddOwn,
            },
            { label: "Add the Hyprland desktop", icon: <Sparkles className="size-4" />, onClick: () => setDialog("hyprland"), hidden: !canAddHyprland },
            {
              label: `Guide to ${distro.catalog?.title.split(" ")[0] ?? "it"}`,
              icon: <BookOpen className="size-4" />,
              onClick: () => onHelp(distro.catalog!.id),
              hidden: !distro.catalog || !guides.some((g) => g.id === distro.catalog!.id),
            },
            { label: "Remove…", icon: <Trash2 className="size-4" />, onClick: () => setDialog("remove"), danger: true },
          ]}
        />
      </div>

      {session && session.state !== "running" && (
        <div className="mt-4">
          {session.state === "failed" ? (
            <Notice tone="bad" icon={<AlertTriangle className="size-4 text-bad" />}>
              <p>{session.message}</p>
              <button className="text-xs underline" onClick={onSessionDismiss}>
                Dismiss
              </button>
            </Notice>
          ) : (
            <Notice icon={<Loader2 className="size-4 animate-spin text-accent" />}>{session.message ?? "Starting"}</Notice>
          )}
        </div>
      )}
      {error && (
        <div className="mt-4">
          <Notice tone="bad">{error}</Notice>
        </div>
      )}
      {notice && (
        <div className="mt-4">
          <Notice>{notice}</Notice>
        </div>
      )}

      <div className="mt-4 flex flex-wrap gap-2">
        <OpenButton items={openItems} disabled={distro.busy || desktopBusy} />
        <Button icon={<FolderOpen className="size-4" />} disabled={distro.busy || distro.info.firstStartPending} onClick={run(() => api.openFiles(distro.name))}>
          Files
        </Button>
      </div>

      {dialog === "remove" && <RemoveDialog distro={distro} onClose={() => setDialog(null)} />}
      {(dialog === "own" || dialog === "hyprland") && <AddDesktopDialog distro={distro} kind={dialog} onClose={() => setDialog(null)} />}
    </Card>
  );
}

/** WSL stopped answering: say so plainly, and offer the restart (which stops every system). */
function NotResponding({ onRestarted }: { onRestarted: () => void }) {
  const [confirm, setConfirm] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  return (
    <>
      <Notice tone="warn" icon={<AlertTriangle className="size-4 text-warn" />}>
        <p className="font-medium">WSL is not responding</p>
        <p className="text-muted">
          Windows' Linux support stopped answering, so Pengi cannot see or start your systems right now. Restarting WSL usually
          fixes it.
        </p>
        <div className="pt-1">
          <Button onClick={() => setConfirm(true)}>Restart WSL…</Button>
        </div>
      </Notice>
      {confirm && (
        <Dialog title="Restart WSL?" onClose={() => setConfirm(false)}>
          <div className="space-y-4">
            <p className="text-muted">
              Every Linux system that is running stops, including open desktops and terminals. Unsaved work in them is lost. Your
              systems and their files stay.
            </p>
            {error && <Notice tone="bad">{error}</Notice>}
            <div className="flex justify-end gap-2">
              <Button onClick={() => setConfirm(false)}>Cancel</Button>
              <Button
                variant="primary"
                busy={busy}
                onClick={async () => {
                  setBusy(true);
                  setError(null);
                  try {
                    await api.restartWsl();
                    setConfirm(false);
                    onRestarted();
                  } catch (e) {
                    setError(String(e));
                  } finally {
                    setBusy(false);
                  }
                }}
              >
                Restart WSL
              </Button>
            </div>
          </div>
        </Dialog>
      )}
    </>
  );
}

export default function MySystems({
  distros, responding, onRestarted, jobs, sessions, onDismissJob, onDismissSession, onGetLinux, onHelp,
}: {
  distros: Distro[] | null;
  responding: boolean;
  onRestarted: () => void;
  jobs: Job[];
  sessions: Record<string, Session>;
  onDismissJob: (id: number) => void;
  onDismissSession: (distro: string) => void;
  onGetLinux: () => void;
  onHelp: (topic: string) => void;
}) {
  // An install shows as its job until it is done; then the system's card takes over.
  const installing = new Set(jobs.filter((j) => j.kind === "install" && j.state === "running").map((j) => j.distro));
  const shown = (distros ?? []).filter((d) => !installing.has(d.name));
  return (
    <div className="space-y-6">
      <header className="flex items-end justify-between gap-4">
        <div>
          <h1 className="font-display text-2xl font-semibold">My systems</h1>
          <p className="text-muted">The Linux systems on this PC.</p>
        </div>
        <Button variant="primary" icon={<Plus className="size-4" />} onClick={onGetLinux}>
          Get Linux
        </Button>
      </header>

      {!responding && <NotResponding onRestarted={onRestarted} />}

      {jobs.length > 0 && (
        <div className="grid gap-4 lg:grid-cols-2">
          {jobs.map((j) => (
            <JobCard key={j.id} job={j} onDismiss={() => onDismissJob(j.id)} />
          ))}
        </div>
      )}

      {distros === null ? (
        <div className="flex items-center gap-2 text-muted">
          <Loader2 className="size-4 animate-spin" /> Looking for your systems
        </div>
      ) : shown.length === 0 && jobs.length === 0 ? (
        <Card className="flex flex-col items-center px-8 py-14 text-center">
          <div className="mb-4 flex size-14 items-center justify-center rounded-2xl bg-accent-soft text-accent">
            <Sparkles className="size-7" />
          </div>
          <h2 className="font-display text-xl font-semibold">No Linux yet</h2>
          <p className="mt-1 max-w-md text-muted">
            Pick a Linux system and Pengi installs it for you. It runs next to Windows, in its own window. Nothing on Windows changes.
          </p>
          <Button variant="primary" className="mt-6" onClick={onGetLinux}>
            Choose your first Linux
          </Button>
        </Card>
      ) : (
        <div className="grid gap-4 lg:grid-cols-2">
          {shown.map((d) => (
            <DistroCard key={d.name} distro={d} session={sessions[d.name]} onSessionDismiss={() => onDismissSession(d.name)} onHelp={onHelp} />
          ))}
        </div>
      )}
    </div>
  );
}
