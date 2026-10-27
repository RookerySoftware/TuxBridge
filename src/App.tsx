import { useEffect, useState } from "react";
import { Boxes, HelpCircle, Loader2, Monitor, PackagePlus, Settings as SettingsIcon, Sparkles } from "lucide-react";
import { api, type Readiness } from "./api";
import { useDistros, useJobs, useSessions } from "./state";
import Setup from "./pages/Setup";
import MySystems from "./pages/MySystems";
import GetLinux from "./pages/GetLinux";
import Settings from "./pages/Settings";
import Docker from "./pages/Docker";
import Help from "./pages/Help";
import Assistant, { useAiStatus } from "./pages/Assistant";
import { onOpen } from "./assistant/store";

type Page = "systems" | "get" | "docker" | "assistant" | "help" | "settings";

const nav: { page: Page; label: string; icon: React.ReactNode; soon?: boolean }[] = [
  { page: "systems", label: "My systems", icon: <Monitor className="size-4" /> },
  { page: "get", label: "Get Linux", icon: <PackagePlus className="size-4" /> },
  { page: "docker", label: "Docker", icon: <Boxes className="size-4" /> },
  { page: "assistant", label: "Assistant", icon: <Sparkles className="size-4" /> },
  { page: "help", label: "Help", icon: <HelpCircle className="size-4" /> },
];

function Main({ readiness }: { readiness: Readiness }) {
  const [page, setPage] = useState<Page>("systems");
  const [topic, setTopic] = useState<string | null>(null);
  const help = (id: string) => {
    setTopic(id);
    setPage("help");
  };
  const { distros, responding, refresh } = useDistros();
  const { jobs, dismiss } = useJobs();
  const { sessions, clear } = useSessions();
  // Known early, so "Ask the assistant" buttons elsewhere know whether it is there.
  useAiStatus();
  useEffect(() => onOpen(() => setPage("assistant")), []);
  // Docker's jobs (turning it on, installing apps) show on its own page.
  const dockerJob = (j: { kind: string }) => j.kind === "docker" || j.kind === "app" || j.kind === "container-update";
  const assistantJob = (j: { kind: string }) => j.kind === "assistant";
  const systemJobs = jobs.filter((j) => !dockerJob(j) && !assistantJob(j));
  const dockerJobs = jobs.filter(dockerJob);
  const assistantJobs = jobs.filter(assistantJob);
  const working = (list: typeof jobs) => list.filter((j) => j.state === "running").length;

  return (
    <div className="flex h-full">
      <aside className="flex w-56 shrink-0 flex-col border-r border-line bg-surface/60 px-3 py-4">
        <div className="mb-6 flex items-center gap-2.5 px-2">
          <div className="flex size-8 items-center justify-center rounded-lg bg-accent font-display font-bold text-accent-ink">E</div>
          <span className="font-display text-base font-semibold">EasyWSL</span>
        </div>
        <nav className="space-y-0.5">
          {nav.map((n) => (
            <button
              key={n.page}
              onClick={() => setPage(n.page)}
              className={`flex w-full items-center gap-2.5 rounded-lg px-2.5 py-2 text-sm transition
                ${page === n.page ? "bg-accent-soft font-medium text-accent" : "text-ink hover:bg-surface-2"}`}
            >
              {n.icon}
              <span className="flex-1 text-left">{n.label}</span>
              {((n.page === "systems" && working(systemJobs) > 0) || (n.page === "docker" && working(dockerJobs) > 0) ||
                (n.page === "assistant" && working(assistantJobs) > 0)) && (
                <Loader2 className="size-3.5 animate-spin text-accent" />
              )}
              {n.soon && <span className="text-[10px] uppercase tracking-wide text-muted">Soon</span>}
            </button>
          ))}
        </nav>
        <div className="mt-auto">
          <button
            onClick={() => setPage("settings")}
            className={`flex w-full items-center gap-2.5 rounded-lg px-2.5 py-2 text-sm transition
              ${page === "settings" ? "bg-accent-soft font-medium text-accent" : "text-ink hover:bg-surface-2"}`}
          >
            <SettingsIcon className="size-4" /> Settings
          </button>
        </div>
      </aside>
      <main className="flex-1 overflow-y-auto px-8 py-7">
        {page === "systems" && (
          <MySystems
            distros={distros}
            responding={responding}
            onRestarted={refresh}
            jobs={systemJobs}
            sessions={sessions}
            onDismissJob={dismiss}
            onDismissSession={clear}
            onGetLinux={() => setPage("get")}
            onHelp={help}
          />
        )}
        {page === "get" && <GetLinux onStarted={() => setPage("systems")} />}
        {page === "docker" && <Docker jobs={dockerJobs} onDismissJob={dismiss} />}
        {page === "assistant" && <Assistant jobs={assistantJobs} onDismissJob={dismiss} onHelp={help} />}
        {page === "help" && <Help topic={topic} onTopic={setTopic} />}
        {page === "settings" && <Settings readiness={readiness} />}
      </main>
    </div>
  );
}

export default function App() {
  const [readiness, setReadiness] = useState<Readiness | null>(null);
  useEffect(() => {
    api.systemCheck().then(setReadiness);
  }, []);

  if (!readiness) {
    return (
      <div className="flex h-full items-center justify-center gap-2 text-muted">
        <Loader2 className="size-4 animate-spin" /> Checking this PC
      </div>
    );
  }
  if (readiness.state !== "ready") return <Setup readiness={readiness} onChange={setReadiness} />;
  return <Main readiness={readiness} />;
}
