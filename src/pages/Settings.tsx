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
        <h2 className="font-display font-semibold">Desktops</h2>
        <Toggle
          checked={settings.fullScreen}
          onChange={(v) => save({ ...settings, fullScreen: v })}
          label="Open full screen"
          description="Borderless, over the whole screen. Ctrl+Alt+Enter, or Full screen in the window's menu (its icon at the top left), switches between full screen and a window at any time."
        />
        <Field
          label="Resolution"
          hint="Automatic uses the screen's full resolution in full screen and the window's size in a window. Another size is scaled to fit the window or screen."
        >
          <select
            className={inputClass}
            value={settings.resolution ?? ""}
            onChange={(e) => save({ ...settings, resolution: e.target.value || null })}
          >
            <option value="">Automatic</option>
            <option value="screen">The screen's resolution, also in a window</option>
            <option value="3840x2160">3840 x 2160 (4K)</option>
            <option value="2560x1440">2560 x 1440</option>
            <option value="1920x1080">1920 x 1080</option>
          </select>
        </Field>
        <Field
          label="Frame rate"
          hint="The most frames a second the desktop sends. For now desktops reach about 50 to 60 a second at 1920 x 1080 and 30 to 40 at 4K, so higher limits mainly help smaller sizes."
        >
          <select className={inputClass} value={settings.frameRate} onChange={(e) => save({ ...settings, frameRate: Number(e.target.value) })}>
            {[60, 120, 144, 240].map((r) => (
              <option key={r} value={r}>
                {r} frames a second
              </option>
            ))}
          </select>
        </Field>
        <p className="text-xs text-muted">Changes apply the next time a desktop opens.</p>
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
