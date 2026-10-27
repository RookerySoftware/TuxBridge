// Settings → AI apps you already use: the EasyWSL skill for Claude Code, Codex and claude.ai.

import { useEffect, useState } from "react";
import { Check, Download, Plus, Trash2 } from "lucide-react";
import { api, type SkillTarget } from "../api";
import { Badge, Button, Card, Notice } from "./ui";

export default function SkillSettings() {
  const [targets, setTargets] = useState<SkillTarget[] | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState<string | null>(null);
  const refresh = () => api.skillsStatus().then(setTargets);
  useEffect(() => {
    refresh();
  }, []);

  const run = async (key: string, action: () => Promise<unknown>) => {
    setBusy(key);
    setError(null);
    try {
      await action();
    } catch (e) {
      setError(String(e));
    }
    setBusy(null);
    refresh();
  };

  return (
    <Card className="space-y-4 p-5">
      <div className="space-y-1">
        <h2 className="font-display font-semibold">AI apps you already use</h2>
        <p className="text-sm text-muted">
          With Claude or ChatGPT you can skip the local assistant: add EasyWSL's skill to your AI app, and it learns how EasyWSL works,
          reads its guides, and can check your systems, logs and Docker for you. Ask it about EasyWSL, or call the skill by name
          (/easywsl in Claude Code, $easywsl in Codex).
        </p>
      </div>
      <div className="divide-y divide-line rounded-xl border border-line">
        {targets?.map((t) => (
          <div key={t.id} className="flex items-center gap-3 px-4 py-3">
            <div className="min-w-0 flex-1">
              <div className="flex items-center gap-2">
                <span className="font-medium">{t.name}</span>
                {t.installed ? <Badge tone="good">Added</Badge> : !t.detected && <Badge>Not found on this PC</Badge>}
              </div>
              <div className="truncate text-xs text-muted" title={t.path}>
                {t.installed ? `In ${t.path}. Kept up to date with EasyWSL.` : `Goes to ${t.path}`}
              </div>
            </div>
            {t.installed ? (
              <Button variant="ghost" icon={<Trash2 className="size-4" />} busy={busy === t.id} onClick={() => run(t.id, () => api.skillRemove(t.id))}>
                Remove
              </Button>
            ) : (
              <Button icon={<Plus className="size-4" />} busy={busy === t.id} onClick={() => run(t.id, () => api.skillInstall(t.id))}>
                Add
              </Button>
            )}
          </div>
        ))}
        <div className="flex items-center gap-3 px-4 py-3">
          <div className="min-w-0 flex-1">
            <div className="font-medium">Claude on the web or Claude Desktop</div>
            <div className="text-xs text-muted">
              {saved ? (
                <span className="inline-flex items-center gap-1 text-good">
                  <Check className="size-3.5" /> Saved as {saved}
                </span>
              ) : (
                "Saves the skill as a .zip. Upload it in Claude under Settings → Capabilities → Skills."
              )}
            </div>
          </div>
          <Button icon={<Download className="size-4" />} busy={busy === "zip"} onClick={() => run("zip", async () => setSaved(await api.skillExport()))}>
            Save .zip
          </Button>
        </div>
      </div>
      {error && <Notice tone="bad">{error}</Notice>}
      <p className="text-xs text-muted">The AI app runs commands only as you allow it there. EasyWSL sends nothing to it; the skill is a few text files.</p>
    </Card>
  );
}
