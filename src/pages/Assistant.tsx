// Ask Pip: the local AI assistant, which speaks as Pip, the mascot. Setting it up (choosing a model size), and the chat.

import { useEffect, useRef, useState } from "react";
import Markdown, { type Components } from "react-markdown";
import remarkGfm from "remark-gfm";
import { openUrl } from "@tauri-apps/plugin-opener";
import { ArrowUp, Check, Copy, Cpu, Loader2, Lock, MessageSquarePlus, Sparkles, Square, WifiOff } from "lucide-react";
import { api, inApp, on, type AiModel, type AiStatus, type Job } from "../api";
import { Badge, Button, Card, Notice } from "../components/ui";
import JobCard from "../components/JobCard";
import Penguin from "../companion/Penguin";
import { newChat, send, setDraft, setInstalled, stop, useAssistant } from "../assistant/store";

export function useAiStatus() {
  const [status, setStatus] = useState<AiStatus | null>(null);
  useEffect(() => {
    const refresh = () =>
      api.aiStatus().then((s) => {
        setStatus(s);
        setInstalled(s.installed);
      });
    refresh();
    return on("ai-changed", refresh);
  }, []);
  return status;
}

function SetUp({ status, busy }: { status: AiStatus; busy: boolean }) {
  const [choice, setChoice] = useState<AiModel["id"]>(status.recommended);
  const [error, setError] = useState<string | null>(null);
  const [starting, setStarting] = useState(false);
  const model = status.models.find((m) => m.id === choice)!;
  return (
    <Card className="space-y-5 p-6">
      <div className="flex items-start gap-4">
        <Penguin size={52} mood="happy" className="shrink-0" />
        <div className="space-y-1">
          <h2 className="font-display text-lg font-semibold">Pip, an AI that knows Linux and EasyWSL</h2>
          <p className="text-muted">
            Ask Pip anything about EasyWSL, Linux, WSL, Windows or Docker, in your own words. Pip knows EasyWSL's guides and sees
            which systems and containers you have, so its answers fit your PC.
          </p>
        </div>
      </div>
      <ul className="grid gap-3 text-sm sm:grid-cols-3">
        <li className="flex gap-2.5">
          <Lock className="mt-0.5 size-4 shrink-0 text-accent" />
          <span><b>Private.</b> It runs on this PC. What you ask never leaves it.</span>
        </li>
        <li className="flex gap-2.5">
          <WifiOff className="mt-0.5 size-4 shrink-0 text-accent" />
          <span><b>Works offline</b> once downloaded. No account, nothing to pay.</span>
        </li>
        <li className="flex gap-2.5">
          <Cpu className="mt-0.5 size-4 shrink-0 text-accent" />
          <span><b>Uses your graphics card</b> when it can, the processor otherwise.</span>
        </li>
      </ul>

      <div className="space-y-2">
        <div className="text-sm font-medium">Size</div>
        <div className="grid gap-2 sm:grid-cols-3">
          {status.models.map((m) => (
            <button
              key={m.id}
              onClick={() => setChoice(m.id)}
              className={`rounded-xl border p-3 text-left transition ${choice === m.id ? "border-accent bg-accent-soft" : "border-line hover:bg-surface-2"}`}
            >
              <div className="flex items-center gap-2">
                <span className="font-medium">{m.name}</span>
                {m.id === status.recommended && <Badge tone="accent">For this PC</Badge>}
              </div>
              <div className="mt-1 text-xs text-muted">{m.description}</div>
              <div className="mt-1.5 text-xs text-muted">{m.sizeGb} GB</div>
            </button>
          ))}
        </div>
        <p className="text-xs text-muted">Found: {status.hardware}. You can change the size later in Settings.</p>
      </div>

      <Notice>
        It downloads {(status.runtimeGb + model.sizeGb).toFixed(1)} GB: Ollama, the program that runs the model, and the model itself
        (Qwen 3.5, free and open). Like any AI it can be wrong, so check commands that change or delete things.
      </Notice>
      {error && <Notice tone="bad">{error}</Notice>}
      <div className="flex justify-end">
        <Button
          variant="primary"
          busy={starting || busy}
          icon={<Sparkles className="size-4" />}
          onClick={async () => {
            setStarting(true);
            setError(null);
            try {
              await api.aiInstall(choice);
            } catch (e) {
              setError(String(e));
            }
            setStarting(false);
          }}
        >
          Install Pip's AI
        </Button>
      </div>
    </Card>
  );
}

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);
  return (
    <button
      aria-label="Copy"
      title="Copy"
      onClick={() => {
        navigator.clipboard.writeText(text).then(() => {
          setCopied(true);
          setTimeout(() => setCopied(false), 1500);
        });
      }}
      className="absolute right-2 top-2 rounded-md bg-surface p-1.5 text-muted opacity-0 transition hover:text-ink group-hover:opacity-100"
    >
      {copied ? <Check className="size-3.5 text-good" /> : <Copy className="size-3.5" />}
    </button>
  );
}

/** The text of a React node tree (a code block's contents, for copying). */
function plain(node: React.ReactNode): string {
  if (typeof node === "string" || typeof node === "number") return String(node);
  if (Array.isArray(node)) return node.map(plain).join("");
  if (node && typeof node === "object" && "props" in node) return plain((node.props as { children?: React.ReactNode }).children);
  return "";
}

function answer(onHelp: (topic: string) => void): Components {
  return {
    h1: ({ children }) => <h3 className="pt-1 font-semibold">{children}</h3>,
    h2: ({ children }) => <h3 className="pt-1 font-semibold">{children}</h3>,
    h3: ({ children }) => <h3 className="pt-1 font-semibold">{children}</h3>,
    p: ({ children }) => <p className="leading-relaxed">{children}</p>,
    ul: ({ children }) => <ul className="list-disc space-y-1 pl-5 leading-relaxed">{children}</ul>,
    ol: ({ children }) => <ol className="list-decimal space-y-1 pl-5 leading-relaxed">{children}</ol>,
    code: ({ className, children }) =>
      className ? (
        <code className="font-mono text-[13px]">{children}</code>
      ) : (
        <code className="rounded bg-surface-2 px-1 py-0.5 font-mono text-[13px]">{children}</code>
      ),
    pre: ({ children }) => (
      <div className="group relative">
        <pre className="overflow-x-auto rounded-lg bg-surface-2 px-4 py-3 pr-10">{children}</pre>
        <CopyButton text={plain(children).replace(/\n$/, "")} />
      </div>
    ),
    table: ({ children }) => (
      <div className="overflow-x-auto">
        <table className="w-full border-collapse text-sm">{children}</table>
      </div>
    ),
    th: ({ children }) => <th className="border-b border-line px-2 py-1.5 text-left font-medium">{children}</th>,
    td: ({ children }) => <td className="border-b border-line px-2 py-1.5 align-top">{children}</td>,
    a: ({ href, children }) => (
      <a
        href={href}
        className="text-accent underline-offset-2 hover:underline"
        onClick={(e) => {
          e.preventDefault();
          if (!href) return;
          if (/^https?:/.test(href)) inApp ? openUrl(href) : window.open(href);
          else onHelp(href.replace(/^\.\//, "").replace(/\.md(#.*)?$/, ""));
        }}
      >
        {children}
      </a>
    ),
  };
}

const SUGGESTIONS = [
  "How do I copy files between Windows and my Linux system?",
  "What can I do with Docker? Give me some ideas.",
  "How do I install a program in Linux?",
  "My desktop window feels slow. What can I try?",
];

function Chat({ status, onHelp }: { status: AiStatus; onHelp: (topic: string) => void }) {
  const { messages, answering, draft } = useAssistant();
  const scroller = useRef<HTMLDivElement>(null);
  const input = useRef<HTMLTextAreaElement>(null);
  const model = status.models.find((m) => m.id === status.model);
  const components = answer(onHelp);

  useEffect(() => {
    const el = scroller.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [messages]);
  useEffect(() => {
    input.current?.focus();
    api.aiWarmUp();
  }, []);

  const submit = () => {
    if (draft.trim() && answering === null) send(draft);
  };

  return (
    <div className="-mx-8 -my-7 flex h-[calc(100%+3.5rem)] flex-col">
      <header className="flex items-center justify-between gap-4 border-b border-line px-8 py-4">
        <div>
          <h1 className="flex items-center gap-2 font-display text-xl font-semibold"><Penguin size={28} /> Ask Pip</h1>
          <p className="text-xs text-muted">
            {model ? `${model.name} model` : "Local model"} · runs on this PC · can be wrong, so check what it suggests
          </p>
        </div>
        <Button variant="ghost" icon={<MessageSquarePlus className="size-4" />} onClick={newChat} disabled={messages.length === 0}>
          New chat
        </Button>
      </header>

      <div ref={scroller} className="flex-1 overflow-y-auto px-8 py-6">
        <div className="mx-auto max-w-3xl space-y-5">
          {messages.length === 0 && (
            <div className="space-y-4 pt-8 text-center">
              <Penguin size={64} mood="happy" className="mx-auto" />
              <div>
                <h2 className="font-display text-lg font-semibold">Hi! What do you want to do?</h2>
                <p className="text-sm text-muted">Ask about EasyWSL, Linux, WSL, Windows or Docker.</p>
              </div>
              <div className="mx-auto grid max-w-2xl gap-2 sm:grid-cols-2">
                {SUGGESTIONS.map((s) => (
                  <button key={s} onClick={() => send(s)} className="rounded-xl border border-line px-3.5 py-2.5 text-left text-sm transition hover:bg-surface-2">
                    {s}
                  </button>
                ))}
              </div>
            </div>
          )}
          {messages.map((m, i) =>
            m.role === "user" ? (
              <div key={i} className="flex justify-end">
                <div className="max-w-[85%] whitespace-pre-wrap rounded-2xl rounded-br-md bg-accent px-4 py-2.5 text-[15px] text-accent-ink">{m.content}</div>
              </div>
            ) : (
              <div key={i} className="flex gap-3">
                <Penguin size={30} className="mt-0.5 shrink-0" mood={answering !== null && i === messages.length - 1 ? "think" : "idle"} />
                <div className="min-w-0 flex-1 space-y-2">
                {m.content ? (
                  <div className="space-y-3 text-[15px]">
                    <Markdown remarkPlugins={[remarkGfm]} components={components}>
                      {m.content}
                    </Markdown>
                  </div>
                ) : answering !== null && i === messages.length - 1 ? (
                  <div className="flex items-center gap-2 text-sm text-muted">
                    <Loader2 className="size-4 animate-spin" />
                    {status.running ? "Thinking" : "Waking Pip up (the first answer takes a little longer)"}
                  </div>
                ) : null}
                {m.error && <Notice tone="bad">{m.error}</Notice>}
                </div>
              </div>
            ),
          )}
        </div>
      </div>

      <div className="border-t border-line px-8 py-4">
        <div className="mx-auto flex max-w-3xl items-end gap-2 rounded-2xl border border-line bg-surface px-3 py-2 focus-within:border-accent">
          <textarea
            ref={input}
            rows={1}
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter" && !e.shiftKey) {
                e.preventDefault();
                submit();
              }
            }}
            placeholder="Ask something. Enter sends, Shift+Enter starts a new line."
            className="max-h-48 flex-1 resize-none bg-transparent py-1.5 text-[15px] outline-none [field-sizing:content] placeholder:text-muted"
          />
          {answering !== null ? (
            <button aria-label="Stop" title="Stop" onClick={stop} className="flex size-8 items-center justify-center rounded-full bg-ink text-surface">
              <Square className="size-3.5 fill-current" />
            </button>
          ) : (
            <button
              aria-label="Send"
              title="Send"
              onClick={submit}
              disabled={!draft.trim()}
              className="flex size-8 items-center justify-center rounded-full bg-accent text-accent-ink transition disabled:opacity-40"
            >
              <ArrowUp className="size-4" />
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

export default function Assistant({ jobs, onDismissJob, onHelp }: { jobs: Job[]; onDismissJob: (id: number) => void; onHelp: (topic: string) => void }) {
  const status = useAiStatus();
  const installing = jobs.some((j) => j.state === "running");
  if (!status) {
    return (
      <div className="flex items-center gap-2 text-muted">
        <Loader2 className="size-4 animate-spin" /> Checking Pip's AI
      </div>
    );
  }
  if (status.installed && !installing) return <Chat status={status} onHelp={onHelp} />;
  return (
    <div className="max-w-3xl space-y-6">
      <header className="space-y-1">
        <h1 className="font-display text-2xl font-semibold">Ask Pip</h1>
        <p className="text-muted">Pip answers questions about EasyWSL, Linux, WSL, Windows and Docker, with an AI on your own PC.</p>
      </header>
      {jobs.map((j) => (
        <JobCard key={j.id} job={j} onDismiss={() => onDismissJob(j.id)} />
      ))}
      {!status.installed && !installing && <SetUp status={status} busy={installing} />}
    </div>
  );
}
