// The Feedback page: report a problem or share an idea, straight to Rookery Software's Discord
// (src-tauri/src/feedback.rs), with an optional diagnostic report; or save it as a file.

import { useEffect, useState } from "react";
import { Bug, FileText, Lightbulb, Loader2, MessageCircle, Save, Send } from "lucide-react";
import { api, type FeedbackKind } from "../api";
import { Button, Card, Dialog, Field, Notice, Toggle, inputClass } from "../components/ui";
import Penguin from "../companion/Penguin";
import { takeDraft } from "../feedback/store";

const KINDS: { id: FeedbackKind; label: string; icon: React.ReactNode; prompt: string; hint: string }[] = [
  {
    id: "bug",
    label: "Something isn't working",
    icon: <Bug className="size-4" />,
    prompt: "What happened? What were you doing just before?",
    hint: "For example: \"Installing Ubuntu stopped at 'Setting up the account' and showed an error.\"",
  },
  {
    id: "idea",
    label: "An idea or request",
    icon: <Lightbulb className="size-4" />,
    prompt: "What would you like TuxBridge to do?",
    hint: "For example: \"It would be great to copy and paste between Windows and the desktop window.\"",
  },
  {
    id: "other",
    label: "Something else",
    icon: <MessageCircle className="size-4" />,
    prompt: "Your message",
    hint: "Anything at all. Pip reads them all.",
  },
];

/** Shortest gap between two sends; the app enforces it too. */
const SEND_GAP = 60;

export default function Feedback() {
  const [kind, setKind] = useState<FeedbackKind>("bug");
  const [message, setMessage] = useState("");
  const [name, setName] = useState("");
  const [includeReport, setIncludeReport] = useState(true);
  const [canSend, setCanSend] = useState<boolean | null>(null);
  const [preview, setPreview] = useState<string | null>(null);
  const [busy, setBusy] = useState<"send" | "save" | "preview" | null>(null);
  const [result, setResult] = useState<{ ok: boolean; text: string } | null>(null);
  const [lastSent, setLastSent] = useState<number | null>(null);
  const [now, setNow] = useState(Date.now());

  useEffect(() => {
    api.feedbackCanSend().then(setCanSend).catch(() => setCanSend(false));
    const draft = takeDraft();
    if (draft) {
      setKind("bug");
      setMessage(draft);
    }
  }, []);

  // The countdown after sending.
  const wait = lastSent ? Math.max(0, SEND_GAP - Math.floor((now - lastSent) / 1000)) : 0;
  useEffect(() => {
    if (!wait) return;
    const t = setInterval(() => setNow(Date.now()), 500);
    return () => clearInterval(t);
  }, [wait]);

  const current = KINDS.find((k) => k.id === kind)!;
  const request = () => ({ kind, message, name, includeReport });
  const empty = !message.trim() && !includeReport;

  const send = async () => {
    setBusy("send");
    setResult(null);
    try {
      await api.feedbackSend(request());
      setResult({ ok: true, text: "Sent. Thank you! Pip carried it straight to the team." });
      setMessage("");
      setLastSent(Date.now());
      setNow(Date.now());
    } catch (e) {
      setResult({ ok: false, text: `Couldn't send: ${String(e)} You can save it as a file instead.` });
    }
    setBusy(null);
  };

  const save = async () => {
    setBusy("save");
    setResult(null);
    try {
      const file = await api.feedbackSave(request());
      setResult({ ok: true, text: `Saved ${file} on your Desktop. Send it to whoever gave you TuxBridge.` });
    } catch (e) {
      setResult({ ok: false, text: String(e) });
    }
    setBusy(null);
  };

  const showReport = async () => {
    setBusy("preview");
    try {
      setPreview(await api.feedbackReport());
    } catch (e) {
      setPreview(`The report couldn't be made: ${String(e)}`);
    }
    setBusy(null);
  };

  return (
    <div className="max-w-2xl space-y-6">
      <header className="flex items-start gap-4">
        <Penguin size={52} mood="happy" />
        <div className="space-y-1">
          <h1 className="font-display text-2xl font-semibold">Feedback</h1>
          <p className="text-muted">
            {canSend === false
              ? "Found a problem, or have an idea? Write it here and save it as a file to send to the developer."
              : "Found a problem, or have an idea? Tell us here: it goes straight to the people who make TuxBridge."}
          </p>
        </div>
      </header>

      <Notice>You're on a test version of TuxBridge, so every report really helps. Thank you!</Notice>

      <Card className="space-y-5 p-5">
        <div className="space-y-2">
          <span className="text-sm font-medium">What's it about?</span>
          <div role="radiogroup" aria-label="What's it about?" className="grid gap-2 sm:grid-cols-3">
            {KINDS.map((k) => (
              <button
                key={k.id}
                type="button"
                role="radio"
                aria-checked={kind === k.id}
                onClick={() => setKind(k.id)}
                className={`flex items-center gap-2 rounded-xl border px-3 py-2.5 text-left text-sm transition ${
                  kind === k.id ? "border-accent bg-accent-soft font-medium text-accent" : "border-line hover:bg-surface-2"
                }`}
              >
                {k.icon}
                {k.label}
              </button>
            ))}
          </div>
        </div>

        <Field label={current.prompt}>
          <textarea
            className={`${inputClass} h-auto min-h-36 resize-y py-2 leading-relaxed`}
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            placeholder={current.hint}
          />
        </Field>

        <Field label="Your name (optional)" hint="So we know who to thank, or who to ask a follow-up question. Leave it empty to stay anonymous.">
          <input className={inputClass} value={name} onChange={(e) => setName(e.target.value)} placeholder="e.g. your Discord name" spellCheck={false} />
        </Field>

        <div className="space-y-2">
          <Toggle
            checked={includeReport}
            onChange={setIncludeReport}
            label="Include a diagnostic report (recommended)"
            description="TuxBridge's and Windows' versions, WSL's version, your Linux systems (names and kinds, not your accounts), Docker's status, and the newest logs. Your Windows user name is hidden."
          />
          <div className="pl-12">
            <Button icon={busy === "preview" ? <Loader2 className="size-4 animate-spin" /> : <FileText className="size-4" />} onClick={showReport} disabled={!includeReport || !!busy}>
              Preview the report
            </Button>
          </div>
        </div>

        <div className="flex flex-wrap items-center gap-2 border-t border-line pt-4">
          {canSend !== false && (
            <Button variant="primary" icon={<Send className="size-4" />} busy={busy === "send"} disabled={!!busy || empty || wait > 0} onClick={send}>
              {wait > 0 ? `Send again in ${wait} s` : "Send"}
            </Button>
          )}
          <Button icon={<Save className="size-4" />} busy={busy === "save"} disabled={!!busy} onClick={save}>
            {canSend === false ? "Save as a file" : "Save as a file instead"}
          </Button>
        </div>
        {result && <Notice tone={result.ok ? "good" : "bad"}>{result.text}</Notice>}
      </Card>

      {preview !== null && (
        <Dialog title="Diagnostic report" onClose={() => setPreview(null)} wide>
          <p className="mb-3 text-sm text-muted">This is exactly what's attached when you send with the report on.</p>
          <pre className="max-h-[60vh] select-text overflow-auto whitespace-pre-wrap rounded-lg bg-surface-2 px-4 py-3 font-mono text-xs leading-relaxed">{preview}</pre>
        </Dialog>
      )}
    </div>
  );
}
