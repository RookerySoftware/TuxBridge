import { useEffect, useState } from "react";
import { api, type Readiness, type Settings as SettingsData } from "../api";
import { Button, Card, Field, Toggle, inputClass } from "../components/ui";

export default function Settings({ readiness }: { readiness: Readiness }) {
  const [settings, setSettings] = useState<SettingsData | null>(null);
  const [defaultDir, setDefaultDir] = useState("");
  const [dir, setDir] = useState("");
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    api.getSettings().then((s) => {
      setSettings(s);
      setDir(s.installDir ?? "");
    });
    api.defaultInstallDir().then(setDefaultDir);
  }, []);

  if (!settings) return null;
  const save = (next: SettingsData) => {
    setSettings(next);
    api.setSettings(next).then(() => {
      setSaved(true);
      setTimeout(() => setSaved(false), 1500);
    });
  };

  return (
    <div className="max-w-2xl space-y-6">
      <header>
        <h1 className="font-display text-2xl font-semibold">Settings</h1>
      </header>

      <Card className="space-y-5 p-5">
        <h2 className="font-display font-semibold">New systems</h2>
        <Field label="Folder for new systems" hint={`Each system gets a folder in here with its disk. Empty: ${defaultDir}`}>
          <div className="flex gap-2">
            <input className={inputClass} value={dir} placeholder={defaultDir} onChange={(e) => setDir(e.target.value)} spellCheck={false} />
            <Button onClick={() => save({ ...settings, installDir: dir.trim() || null })}>Save</Button>
          </div>
        </Field>
      </Card>

      <Card className="space-y-5 p-5">
        <h2 className="font-display font-semibold">Hyprland desktop</h2>
        <Toggle
          checked={settings.fullScreen}
          onChange={(v) => save({ ...settings, fullScreen: v })}
          label="Open full screen"
          description="Only full screen do the Windows key and Alt+Tab reach Linux. Ctrl+Alt+Enter switches between full screen and a window."
        />
      </Card>

      <Card className="space-y-1 p-5 text-sm text-muted">
        <h2 className="mb-2 font-display font-semibold text-ink">About</h2>
        <p>EasyWSL 0.1.0 (pre-release)</p>
        <p>WSL {readiness.state === "ready" ? readiness.wslVersion : "not ready"}</p>
        <p>The Hyprland desktop comes from hyprland-wsl.</p>
      </Card>
      {saved && <p className="text-sm text-good">Saved.</p>}
    </div>
  );
}
