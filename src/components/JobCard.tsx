// A running or finished job (installing a system, adding Hyprland): its steps with their state,
// what the tools print right now, and on failure the reason and the log.

import { AlertTriangle, Check, Circle, FileText, Loader2, Sparkles, X } from "lucide-react";
import { api, type Job } from "../api";
import { askAbout, useAssistant } from "../assistant/store";
import { Button, Card, IconButton } from "./ui";

function StepIcon({ state }: { state: Job["steps"][number]["state"] }) {
  switch (state) {
    case "done":
      return <Check className="size-4 text-good" />;
    case "running":
      return <Loader2 className="size-4 animate-spin text-accent" />;
    case "failed":
      return <X className="size-4 text-bad" />;
    default:
      return <Circle className="size-3.5 text-line" />;
  }
}

/** Opens the assistant with the failure, the step and the end of the log. */
async function ask(job: Job) {
  const step = job.steps.find((s) => s.state === "failed")?.title;
  const log = await api.jobLogTail(job.id).catch(() => "");
  const fence = "```";
  askAbout(
    `"${job.title}" did not finish in Pengi${step ? `, at the step "${step}"` : ""}. It said: ${job.error ?? "no reason given"}` +
      (log ? `\n\nThe end of its log:\n\n${fence}\n${log}\n${fence}` : "") +
      "\n\nWhat went wrong, and how do I fix it?",
  );
}

export default function JobCard({ job, onDismiss }: { job: Job; onDismiss: () => void }) {
  const { installed: assistant } = useAssistant();
  const done = job.state === "done";
  const failed = job.state === "failed";
  return (
    <Card className="p-5 animate-in">
      <div className="flex items-start justify-between gap-3">
        <div>
          <h3 className="font-display text-base font-semibold">{job.title}</h3>
          <p className="text-sm text-muted">
            {done
              ? job.kind === "docker"
                ? "Docker is on."
                : job.kind === "app"
                  ? "It is running."
                  : job.kind === "update" || job.kind === "container-update"
                    ? "It is up to date."
                    : `${job.distro} is ready.`
              : failed
                ? "This did not finish."
                : job.kind === "install" || job.kind === "desktop"
                  ? `As "${job.distro}". You can keep using your PC meanwhile.`
                  : "You can keep using your PC meanwhile."}
          </p>
        </div>
        {!(job.state === "running") && (
          <IconButton label="Dismiss" onClick={onDismiss}>
            <X className="size-4" />
          </IconButton>
        )}
      </div>

      <ol className="mt-4 space-y-2.5">
        {job.steps.map((s) => (
          <li key={s.title} className="flex items-start gap-3">
            <span className="mt-0.5 flex size-5 items-center justify-center">
              <StepIcon state={s.state} />
            </span>
            <div className="min-w-0 flex-1">
              <div className={`text-sm ${s.state === "pending" ? "text-muted" : "font-medium"}`}>
                {s.title}
                {s.estimate && s.state !== "done" && <span className="ml-2 text-xs font-normal text-muted">about {s.estimate}</span>}
              </div>
              {s.state === "running" && job.lastLine && (
                <div className="mt-0.5 truncate font-mono text-xs text-muted" title={job.lastLine}>
                  {job.lastLine}
                </div>
              )}
            </div>
          </li>
        ))}
      </ol>

      {failed && (
        <div className="mt-4 flex gap-2.5 rounded-lg bg-bad-soft px-3.5 py-3 text-sm">
          <AlertTriangle className="mt-0.5 size-4 shrink-0 text-bad" />
          <div>
            <p>{job.error}</p>
            {job.kind === "install" && <p className="mt-1 text-muted">The half-installed system was removed, so trying again starts clean.</p>}
          </div>
        </div>
      )}
      {(failed || done) && (
        <div className="mt-4 flex flex-wrap gap-2">
          {failed && assistant && job.kind !== "assistant" && (
            <Button icon={<Sparkles className="size-4" />} onClick={() => ask(job)}>
              Ask Pip
            </Button>
          )}
          <Button variant="ghost" icon={<FileText className="size-4" />} onClick={() => api.reveal(job.log)}>
            Show the log file
          </Button>
        </div>
      )}
    </Card>
  );
}
