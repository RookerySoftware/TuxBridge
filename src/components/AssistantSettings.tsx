// Settings → Assistant: its size, and removing it.

import { useState } from "react";
import { Sparkles, Trash2 } from "lucide-react";
import { api, type AiModel } from "../api";
import { openAssistant } from "../assistant/store";
import { useAiStatus } from "../pages/Assistant";
import { Badge, Button, Card, Dialog, Notice } from "./ui";

export default function AssistantSettings() {
  const status = useAiStatus();
  const [choice, setChoice] = useState<AiModel["id"] | null>(null);
  const [confirm, setConfirm] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  if (!status) return null;
  const current = status.model;
  const picked = choice ?? current ?? status.recommended;

  return (
    <Card className="space-y-4 p-5">
      <h2 className="font-display font-semibold">Ask Pip (the AI)</h2>
      {!status.installed ? (
        <div className="flex items-center justify-between gap-4">
          <p className="text-sm text-muted">Not installed. It is an AI on this PC that answers questions about EasyWSL, Linux, WSL, Windows and Docker.</p>
          <Button icon={<Sparkles className="size-4" />} onClick={openAssistant}>
            Set it up
          </Button>
        </div>
      ) : (
        <>
          <div className="space-y-2">
            <div className="text-sm font-medium">Size</div>
            <div className="grid gap-2 sm:grid-cols-3">
              {status.models.map((m) => (
                <button
                  key={m.id}
                  onClick={() => setChoice(m.id)}
                  className={`rounded-xl border p-3 text-left text-sm transition ${picked === m.id ? "border-accent bg-accent-soft" : "border-line hover:bg-surface-2"}`}
                >
                  <div className="flex flex-wrap items-center gap-1.5">
                    <span className="font-medium">{m.name}</span>
                    {m.id === current && <Badge tone="good">Installed</Badge>}
                    {m.id === status.recommended && m.id !== current && <Badge tone="accent">For this PC</Badge>}
                  </div>
                  <div className="mt-1 text-xs text-muted">{m.sizeGb} GB</div>
                </button>
              ))}
            </div>
            <p className="text-xs text-muted">{status.models.find((m) => m.id === picked)?.description} Found: {status.hardware}.</p>
          </div>
          {error && <Notice tone="bad">{error}</Notice>}
          <div className="flex flex-wrap justify-between gap-2">
            <Button variant="ghost" icon={<Trash2 className="size-4" />} onClick={() => setConfirm(true)}>
              Remove Pip's AI
            </Button>
            {picked !== current && (
            <Button
              variant="primary"
              busy={busy}
              onClick={async () => {
                setBusy(true);
                setError(null);
                try {
                  await api.aiInstall(picked);
                  openAssistant();
                } catch (e) {
                  setError(String(e));
                }
                setBusy(false);
              }}
            >
              Change to {status.models.find((m) => m.id === picked)?.name}
            </Button>
            )}
          </div>
        </>
      )}
      {confirm && (
        <Dialog title="Remove Pip's AI?" onClose={() => setConfirm(false)}>
          <div className="space-y-4">
            <p className="text-muted">This deletes Ollama and the model from this PC, freeing several GB. Your chat is kept. You can install it again later.</p>
            {error && <Notice tone="bad">{error}</Notice>}
            <div className="flex justify-end gap-2">
              <Button onClick={() => setConfirm(false)}>Keep it</Button>
              <Button
                variant="danger"
                busy={busy}
                onClick={async () => {
                  setBusy(true);
                  setError(null);
                  try {
                    await api.aiRemove();
                    setConfirm(false);
                  } catch (e) {
                    setError(String(e));
                  }
                  setBusy(false);
                }}
              >
                Remove
              </Button>
            </div>
          </div>
        </Dialog>
      )}
    </Card>
  );
}
