// Docker: turning TuxBridge's engine on, its containers (and Docker Desktop's), and the app gallery.

import { useCallback, useEffect, useMemo, useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  Boxes, Brush, Download, ExternalLink, FileText, Loader2, Play, Plus, Power, RotateCw, Square, SquareTerminal, Trash2,
} from "lucide-react";
import { api, inApp, on, type Container, type DiskUse, type Distro, type DockerStatus, type GalleryApp, type Job } from "../api";
import { Badge, Button, Card, Dialog, Field, IconButton, Menu, Monogram, Notice, Toggle, inputClass } from "../components/ui";
import JobCard from "../components/JobCard";

const open = (url: string) => (inApp ? openUrl(url) : window.open(url));

/** The engine's state and the containers, refreshed by the backend's events and a slow poll. */
function useDocker() {
  const [status, setStatus] = useState<DockerStatus | null>(null);
  const [containers, setContainers] = useState<Container[] | null>(null);
  const refresh = useCallback(async () => {
    const s = await api.dockerStatus();
    setStatus(s);
    setContainers(s.running || s.desktopRunning ? await api.containers() : []);
  }, []);
  useEffect(() => {
    refresh();
    const off = on("docker-changed", refresh);
    const timer = setInterval(refresh, 5000);
    return () => {
      off();
      clearInterval(timer);
    };
  }, [refresh]);
  return { status, containers, refresh };
}

function TurnOn({ desktopInstalled }: { desktopInstalled: boolean }) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  return (
    <Card className="space-y-4 p-6">
      <div className="flex items-start gap-4">
        <div className="flex size-12 shrink-0 items-center justify-center rounded-xl bg-accent-soft text-accent">
          <Boxes className="size-6" />
        </div>
        <div className="space-y-1.5">
          <h2 className="font-display text-lg font-semibold">Turn on Docker</h2>
          <p className="text-sm text-muted">
            Docker runs apps in containers: each app comes with everything it needs, so it installs in one step and never mixes with
            the rest of your PC. TuxBridge sets up the free Docker Engine in a small hidden Linux system of its own.
          </p>
        </div>
      </div>
      <ul className="ml-16 list-disc space-y-1 pl-4 text-sm text-muted">
        <li>
          <span className="text-ink">The app gallery</span>: media server, cloud storage, AI chat, databases and more, in a click.
        </li>
        <li>
          The <code className="rounded bg-surface-2 px-1 text-ink">docker</code> command works in PowerShell and Windows Terminal, as
          in every guide (with <code className="rounded bg-surface-2 px-1 text-ink">docker compose</code>).
        </li>
        <li>About 250 MB to download, 2 to 4 minutes. Free, also for work (no Docker Desktop licence needed).</li>
      </ul>
      {desktopInstalled && (
        <div className="ml-16">
          <Notice>
            Docker Desktop is installed too. Both can be used: its containers show here as well, and the <code>docker</code> command
            keeps using Docker Desktop unless you switch with <code>docker context use easywsl</code>.
          </Notice>
        </div>
      )}
      {error && <Notice tone="bad">{error}</Notice>}
      <div className="ml-16">
        <Button
          variant="primary"
          busy={busy}
          icon={<Power className="size-4" />}
          onClick={async () => {
            setBusy(true);
            setError(null);
            try {
              await api.dockerEnable();
            } catch (e) {
              setError(String(e));
              setBusy(false);
            }
          }}
        >
          Turn on Docker
        </Button>
      </div>
    </Card>
  );
}

function CleanUpDialog({ onClose }: { onClose: () => void }) {
  const [disk, setDisk] = useState<DiskUse[] | null>(null);
  const [images, setImages] = useState(false);
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const load = useCallback(() => api.dockerDisk().then(setDisk).catch((e) => setError(String(e))), []);
  useEffect(() => {
    load();
  }, [load]);
  const names: Record<string, string> = { Images: "Images (downloaded apps)", Containers: "Containers", "Local Volumes": "Volumes (apps' data)", "Build Cache": "Build cache" };
  return (
    <Dialog title="Disk use and clean-up" onClose={onClose}>
      <div className="space-y-4">
        {!disk && !error && <Loader2 className="size-4 animate-spin text-muted" />}
        {disk && (
          <table className="w-full text-sm">
            <thead>
              <tr className="text-left text-xs text-muted">
                <th className="pb-1.5 font-medium">What</th>
                <th className="pb-1.5 font-medium">Size</th>
                <th className="pb-1.5 font-medium">Could be freed</th>
              </tr>
            </thead>
            <tbody>
              {disk.map((d) => (
                <tr key={d.kind} className="border-t border-line">
                  <td className="py-1.5">{names[d.kind] ?? d.kind}</td>
                  <td className="py-1.5">{d.size}</td>
                  <td className="py-1.5 text-muted">{d.reclaimable}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
        <p className="text-sm text-muted">
          Clean-up removes stopped containers, leftover networks and build files, and images nothing uses. Apps' data (volumes) is
          always kept.
        </p>
        <Toggle
          checked={images}
          onChange={setImages}
          label="Also remove images of stopped apps"
          description="Frees the most. A removed image downloads again when its app starts."
        />
        {result && <Notice tone="good">{result}</Notice>}
        {error && <Notice tone="bad">{error}</Notice>}
        <div className="flex justify-end gap-2">
          <Button onClick={onClose}>Close</Button>
          <Button
            variant="primary"
            busy={busy}
            icon={<Brush className="size-4" />}
            onClick={async () => {
              setBusy(true);
              setError(null);
              try {
                setResult(await api.dockerCleanUp(images));
                await load();
              } catch (e) {
                setError(String(e));
              }
              setBusy(false);
            }}
          >
            Clean up
          </Button>
        </div>
      </div>
    </Dialog>
  );
}

function RemoveDockerDialog({ onClose }: { onClose: () => void }) {
  const [typed, setTyped] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  return (
    <Dialog title="Remove Docker?" onClose={onClose}>
      <div className="space-y-4">
        <p className="text-muted">
          This deletes TuxBridge's Docker engine and <b className="text-ink">everything in it</b>: every container, image and app's data.
          It cannot be undone. Docker Desktop, if you have it, is not touched.
        </p>
        <Field label="Type docker to confirm">
          <input className={inputClass} value={typed} onChange={(e) => setTyped(e.target.value)} spellCheck={false} />
        </Field>
        {error && <Notice tone="bad">{error}</Notice>}
        <div className="flex justify-end gap-2">
          <Button onClick={onClose}>Keep it</Button>
          <Button
            variant="danger"
            busy={busy}
            disabled={typed.trim().toLowerCase() !== "docker"}
            onClick={async () => {
              setBusy(true);
              try {
                await api.dockerRemove();
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

function Engine({ status, onChanged }: { status: DockerStatus; onChanged: () => void }) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [dialog, setDialog] = useState<"clean" | "remove" | null>(null);
  const toggle = async () => {
    setBusy(true);
    setError(null);
    try {
      await (status.running ? api.dockerStop() : api.dockerStart());
    } catch (e) {
      setError(String(e));
    }
    setBusy(false);
    onChanged();
  };
  return (
    <Card className="p-5">
      <div className="flex flex-wrap items-center gap-4">
        <div className="flex size-11 shrink-0 items-center justify-center rounded-xl bg-accent-soft text-accent">
          <Boxes className="size-5" />
        </div>
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <h2 className="font-display text-base font-semibold">Docker Engine</h2>
            {status.running ? <Badge tone="good">Running</Badge> : <Badge>Stopped</Badge>}
          </div>
          <p className="text-sm text-muted">
            {status.running ? `Version ${status.version}. ` : "Containers are stopped too. "}
            In a terminal: <code className="rounded bg-surface-2 px-1 text-ink">docker run hello-world</code>
          </p>
        </div>
        <Button variant={status.running ? "secondary" : "primary"} busy={busy} icon={status.running ? <Square className="size-4" /> : <Play className="size-4" />} onClick={toggle}>
          {status.running ? "Stop" : "Start"}
        </Button>
        <Menu
          items={[
            { label: "Disk use and clean-up", icon: <Brush className="size-4" />, onClick: () => setDialog("clean"), hidden: !status.running },
            { label: "Remove Docker", icon: <Trash2 className="size-4" />, onClick: () => setDialog("remove"), danger: true },
          ]}
        />
      </div>
      {error && (
        <div className="mt-3">
          <Notice tone="bad">{error}</Notice>
        </div>
      )}
      {dialog === "clean" && <CleanUpDialog onClose={() => setDialog(null)} />}
      {dialog === "remove" && <RemoveDockerDialog onClose={() => { setDialog(null); onChanged(); }} />}
    </Card>
  );
}

/** docker inside the user's Linux systems, pointed at this engine. */
function InSystems({ busy }: { busy: string[] }) {
  const [distros, setDistros] = useState<Distro[]>([]);
  const [error, setError] = useState<string | null>(null);
  const load = useCallback(() => api.listDistros().then((l) => setDistros(l.distros)), []);
  useEffect(() => {
    load();
    return on("distros-changed", load);
  }, [load]);
  if (distros.length === 0) return null;
  return (
    <Card className="space-y-3 p-5">
      <div>
        <h2 className="font-display text-base font-semibold">docker in your Linux systems</h2>
        <p className="text-sm text-muted">
          Set up a system, and <code className="rounded bg-surface-2 px-1 text-ink">docker</code> and{" "}
          <code className="rounded bg-surface-2 px-1 text-ink">docker compose</code> in its terminal use this engine too: the same
          containers everywhere.
        </p>
      </div>
      <div className="divide-y divide-line">
        {distros.map((d) => (
          <div key={d.name} className="flex items-center justify-between gap-3 py-2">
            <span className="text-sm">{d.name}</span>
            {d.info.docker ? (
              <Badge tone="good">Set up</Badge>
            ) : (
              <Button
                busy={busy.includes(d.name)}
                disabled={d.busy}
                onClick={async () => {
                  setError(null);
                  try {
                    await api.dockerUseIn(d.name);
                  } catch (e) {
                    setError(String(e));
                  }
                }}
              >
                Set up
              </Button>
            )}
          </div>
        ))}
      </div>
      {error && <Notice tone="bad">{error}</Notice>}
    </Card>
  );
}

function LogsDialog({ container, onClose }: { container: Container; onClose: () => void }) {
  const [text, setText] = useState<string | null>(null);
  const load = useCallback(() => {
    api.containerLogs(container.engine, container.id).then(setText).catch((e) => setText(String(e)));
  }, [container]);
  useEffect(() => {
    load();
  }, [load]);
  return (
    <Dialog title={`Log of ${container.name}`} onClose={onClose} wide>
      <div className="space-y-3">
        <pre className="max-h-[60vh] overflow-auto whitespace-pre-wrap break-all rounded-lg bg-surface-2 p-3 font-mono text-xs">
          {text === null ? "Loading..." : text.trim() || "Nothing logged yet."}
        </pre>
        <div className="flex justify-end gap-2">
          <Button icon={<RotateCw className="size-4" />} onClick={load}>
            Refresh
          </Button>
          <Button onClick={onClose}>Close</Button>
        </div>
      </div>
    </Dialog>
  );
}

function RemoveContainerDialog({ container, onClose }: { container: Container; onClose: () => void }) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  return (
    <Dialog title={`Remove ${container.name}?`} onClose={onClose}>
      <div className="space-y-4">
        <p className="text-muted">
          The container is stopped and removed. {container.app ? "Its data is kept, so installing the app again picks up where it left off." : "Data it kept in volumes stays."}
        </p>
        {error && <Notice tone="bad">{error}</Notice>}
        <div className="flex justify-end gap-2">
          <Button onClick={onClose}>Keep it</Button>
          <Button
            variant="danger"
            busy={busy}
            onClick={async () => {
              setBusy(true);
              try {
                await api.containerAction(container.engine, container.id, "remove");
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

function ContainerRow({ container, app, updating }: { container: Container; app: GalleryApp | undefined; updating: boolean }) {
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [dialog, setDialog] = useState<"logs" | "remove" | null>(null);
  const running = container.state === "running";
  const act = async (action: "start" | "stop" | "restart") => {
    setBusy(action);
    setError(null);
    try {
      await api.containerAction(container.engine, container.id, action);
    } catch (e) {
      setError(String(e));
    }
    setBusy(null);
  };
  const update = async () => {
    setError(null);
    try {
      await api.updateContainer(container.engine, container.id, container.name);
    } catch (e) {
      setError(String(e));
    }
  };
  // Web pages open in the browser; a database's port is only shown.
  const web = app ? app.web : true;
  return (
    <Card className="p-4">
      <div className="flex items-center gap-3.5">
        {app ? (
          <Monogram title={app.name} accent={app.accent} />
        ) : (
          <div className="flex size-11 shrink-0 items-center justify-center rounded-xl bg-surface-2 text-muted">
            <Boxes className="size-5" />
          </div>
        )}
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-2">
            <span className="font-medium">{app?.name ?? container.name}</span>
            <span className={`size-2 rounded-full ${running ? "bg-good" : container.state === "restarting" ? "bg-warn" : "bg-line"}`} aria-hidden />
            <span className="text-xs text-muted">{container.status}</span>
            {container.engine === "desktop" && <Badge>Docker Desktop</Badge>}
            {updating && (
              <Badge tone="accent">
                <Loader2 className="size-3 animate-spin" /> Updating
              </Badge>
            )}
          </div>
          <div className="truncate text-xs text-muted" title={container.image}>
            {app ? `${container.name} · ` : ""}
            {container.image}
          </div>
        </div>
        {running &&
          container.ports.slice(0, 2).map((p) =>
            web ? (
              <Button key={p} icon={<ExternalLink className="size-4" />} onClick={() => open(`http://localhost:${p}`)}>
                Open
                <span className="text-xs font-normal text-muted">:{p}</span>
              </Button>
            ) : (
              <span key={p} className="rounded-lg bg-surface-2 px-2.5 py-1.5 font-mono text-xs" title="Connect your programs here">
                localhost:{p}
              </span>
            ),
          )}
        {running ? (
          <IconButton label="Stop" onClick={() => act("stop")} disabled={!!busy || updating}>
            {busy === "stop" ? <Loader2 className="size-4 animate-spin" /> : <Square className="size-4" />}
          </IconButton>
        ) : (
          <IconButton label="Start" onClick={() => act("start")} disabled={!!busy || updating}>
            {busy === "start" ? <Loader2 className="size-4 animate-spin" /> : <Play className="size-4" />}
          </IconButton>
        )}
        <Menu
          items={[
            { label: "Update", icon: <Download className="size-4" />, onClick: update, hidden: updating },
            { label: "Restart", icon: <RotateCw className="size-4" />, onClick: () => act("restart"), hidden: !running || updating },
            { label: "Show its log", icon: <FileText className="size-4" />, onClick: () => setDialog("logs") },
            { label: "Remove", icon: <Trash2 className="size-4" />, onClick: () => setDialog("remove"), danger: true, hidden: updating },
          ]}
        />
      </div>
      {app && running && busy === null && !error && (
        <p className="mt-2 pl-[58px] text-xs text-muted">{app.after}</p>
      )}
      {error && (
        <div className="mt-3">
          <Notice tone="bad">{error}</Notice>
        </div>
      )}
      {dialog === "logs" && <LogsDialog container={container} onClose={() => setDialog(null)} />}
      {dialog === "remove" && <RemoveContainerDialog container={container} onClose={() => setDialog(null)} />}
    </Card>
  );
}

function InstallDialog({ app, onClose, onStarted }: { app: GalleryApp; onClose: () => void; onStarted: () => void }) {
  const [answers, setAnswers] = useState<Record<string, string>>(() => Object.fromEntries(app.questions.map((q) => [q.id, q.default ?? ""])));
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const missing = app.questions.some((q) => q.required && !answers[q.id]?.trim());
  return (
    <Dialog title={`Install ${app.name}`} onClose={onClose}>
      <div className="space-y-4">
        <p className="text-sm text-muted">{app.description}</p>
        {app.note && <Notice>{app.note}</Notice>}
        {app.questions.map((q) => (
          <Field key={q.id} label={q.label} hint={q.hint ?? undefined}>
            <input
              className={inputClass}
              type={q.kind === "password" ? "password" : "text"}
              value={answers[q.id] ?? ""}
              onChange={(e) => setAnswers({ ...answers, [q.id]: e.target.value })}
              spellCheck={false}
              autoComplete={q.kind === "password" ? "new-password" : "off"}
            />
          </Field>
        ))}
        <p className="text-xs text-muted">
          Downloads {app.image} ({app.download}) and starts it. It starts again with Docker, until you stop it.
        </p>
        {error && <Notice tone="bad">{error}</Notice>}
        <div className="flex justify-end gap-2">
          <Button onClick={onClose}>Cancel</Button>
          <Button
            variant="primary"
            busy={busy}
            disabled={missing}
            icon={<Plus className="size-4" />}
            onClick={async () => {
              setBusy(true);
              setError(null);
              try {
                await api.installApp(app.id, answers);
                onStarted();
                onClose();
              } catch (e) {
                setError(String(e));
                setBusy(false);
              }
            }}
          >
            Install
          </Button>
        </div>
      </div>
    </Dialog>
  );
}

function Gallery({ apps, containers, running, onStarted }: { apps: GalleryApp[]; containers: Container[]; running: boolean; onStarted: () => void }) {
  const [installing, setInstalling] = useState<GalleryApp | null>(null);
  const categories = useMemo(() => [...new Set(apps.map((a) => a.category))], [apps]);
  return (
    <div className="space-y-6">
      {!running && <Notice>Start Docker to install apps.</Notice>}
      {categories.map((category) => (
        <section key={category} className="space-y-2.5">
          <h3 className="text-xs font-semibold uppercase tracking-wide text-muted">{category}</h3>
          <div className="grid gap-3 lg:grid-cols-2">
            {apps
              .filter((a) => a.category === category)
              .map((a) => {
                const installed = containers.find((c) => c.app === a.id && c.engine === "easywsl");
                return (
                  <Card key={a.id} className="flex items-start gap-3.5 p-4">
                    <Monogram title={a.name} accent={a.accent} />
                    <div className="min-w-0 flex-1 space-y-1">
                      <div className="flex items-center gap-2">
                        <span className="font-medium">{a.name}</span>
                        {installed && <Badge tone="good">Installed</Badge>}
                      </div>
                      <p className="text-sm text-muted">{a.tagline}</p>
                      <div className="flex gap-2 pt-1.5">
                        {installed ? (
                          installed.state === "running" && a.web && installed.ports[0] ? (
                            <Button icon={<ExternalLink className="size-4" />} onClick={() => open(`http://localhost:${installed.ports[0]}`)}>
                              Open
                            </Button>
                          ) : null
                        ) : (
                          <Button variant="primary" disabled={!running} icon={<Plus className="size-4" />} onClick={() => setInstalling(a)}>
                            Install
                          </Button>
                        )}
                        <Button variant="ghost" onClick={() => open(a.website)}>
                          Website
                        </Button>
                      </div>
                    </div>
                  </Card>
                );
              })}
          </div>
        </section>
      ))}
      {installing && <InstallDialog app={installing} onClose={() => setInstalling(null)} onStarted={onStarted} />}
    </div>
  );
}

export default function Docker({ jobs, onDismissJob }: { jobs: Job[]; onDismissJob: (id: number) => void }) {
  const { status, containers: listed, refresh } = useDocker();
  const containers = listed ?? [];
  const [apps, setApps] = useState<GalleryApp[]>([]);
  const [tab, setTab] = useState<"containers" | "apps">("containers");
  useEffect(() => {
    api.apps().then(setApps);
  }, []);
  // A finished job changes what is installed.
  const finished = jobs.filter((j) => j.state !== "running").length;
  useEffect(() => {
    refresh();
  }, [finished, refresh]);
  const enabling = jobs.some((j) => j.kind === "docker" && j.state === "running");

  return (
    <div className="max-w-4xl space-y-6">
      <header className="space-y-1">
        <h1 className="font-display text-2xl font-semibold">Docker</h1>
        <p className="text-muted">Apps in containers: installed in a click, kept apart from everything else.</p>
      </header>

      {jobs.map((j) => (
        <JobCard key={j.id} job={j} onDismiss={() => onDismissJob(j.id)} />
      ))}

      {!status && (
        <div className="flex items-center gap-2 text-muted">
          <Loader2 className="size-4 animate-spin" /> Checking Docker
        </div>
      )}
      {status && !status.installed && !enabling && <TurnOn desktopInstalled={status.desktopInstalled} />}
      {status?.installed && (
        <>
          <Engine status={status} onChanged={refresh} />
          {status.desktopRunning && (
            <Notice>
              Docker Desktop is running too; its containers are listed with a Docker Desktop badge.
            </Notice>
          )}
          <div className="flex gap-1 border-b border-line">
            {(
              [
                ["containers", `Containers${containers.length ? ` (${containers.length})` : ""}`],
                ["apps", "App gallery"],
              ] as const
            ).map(([id, label]) => (
              <button
                key={id}
                onClick={() => setTab(id)}
                className={`-mb-px border-b-2 px-3 py-2 text-sm transition ${tab === id ? "border-accent font-medium text-accent" : "border-transparent text-muted hover:text-ink"}`}
              >
                {label}
              </button>
            ))}
          </div>
          {tab === "containers" && listed === null && <Loader2 className="size-4 animate-spin text-muted" />}
          {tab === "containers" &&
            listed !== null &&
            (containers.length === 0 ? (
              <Card className="space-y-3 p-6 text-sm">
                <p className="font-medium">No containers yet.</p>
                <p className="text-muted">
                  Install one from the app gallery, or start one in a terminal, for example a web server on port 8080:
                </p>
                <code className="block rounded-lg bg-surface-2 px-3 py-2 font-mono text-xs">docker run -d -p 8080:80 --name web nginx</code>
                <div className="flex gap-2">
                  <Button variant="primary" onClick={() => setTab("apps")}>
                    Browse apps
                  </Button>
                  <Button variant="ghost" icon={<SquareTerminal className="size-4" />} onClick={() => open("https://docs.docker.com/get-started/")}>
                    Docker's guide
                  </Button>
                </div>
              </Card>
            ) : (
              <div className="space-y-2.5">
                {containers.map((c) => (
                  <ContainerRow
                    key={`${c.engine}-${c.id}`}
                    container={c}
                    app={c.engine === "easywsl" ? apps.find((a) => a.id === c.app) : undefined}
                    updating={jobs.some((j) => j.kind === "container-update" && j.state === "running" && j.distro === `container ${c.name}`)}
                  />
                ))}
              </div>
            ))}
          {tab === "apps" && <Gallery apps={apps} containers={containers} running={status.running} onStarted={refresh} />}
          {tab === "containers" && status.running && (
            <InSystems busy={jobs.filter((j) => j.kind === "docker" && j.state === "running").map((j) => j.distro)} />
          )}
        </>
      )}
    </div>
  );
}
