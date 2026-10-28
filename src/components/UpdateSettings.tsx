// Settings → Update TuxBridge: checks tuxbridge.com for a newer version when Settings opens, and
// installs it (src-tauri/src/updates.rs). Installing closes TuxBridge; its installer starts it again.

import { useEffect, useState } from "react";
import Markdown from "react-markdown";
import { Download, Loader2, RefreshCw } from "lucide-react";
import { api, on, type UpdateInfo } from "../api";
import { Button, Card, Notice } from "./ui";

export default function UpdateSettings() {
  const [info, setInfo] = useState<UpdateInfo | null>(null);
  const [checking, setChecking] = useState(false);
  const [installing, setInstalling] = useState(false);
  const [progress, setProgress] = useState<[number, number | null] | null>(null);
  const [error, setError] = useState<string | null>(null);

  const check = async () => {
    setChecking(true);
    setError(null);
    try {
      setInfo(await api.updateCheck());
    } catch (e) {
      setError(String(e));
    }
    setChecking(false);
  };

  useEffect(() => {
    check();
    return on<[number, number | null]>("update-progress", setProgress);
  }, []);

  const install = async () => {
    setInstalling(true);
    setError(null);
    setProgress(null);
    try {
      await api.updateInstall();
      // On Windows TuxBridge closes here while the installer runs; this only shows if it doesn't.
    } catch (e) {
      setError(String(e));
      setInstalling(false);
    }
  };

  const percent = progress && progress[1] ? Math.min(100, Math.round((progress[0] / progress[1]) * 100)) : null;
  const mb = (n: number) => (n / 1024 / 1024).toFixed(1);

  return (
    <Card className="space-y-4 p-5">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h2 className="font-display font-semibold">Update TuxBridge</h2>
          <p className="text-sm text-muted">
            {info ? `You have version ${info.current}.` : "Checking which version you have…"}{" "}
            {info && !info.available && !checking && "It's the newest one."}
          </p>
        </div>
        {!installing && (
          <Button
            icon={checking ? <Loader2 className="size-4 animate-spin" /> : <RefreshCw className="size-4" />}
            onClick={check}
            disabled={checking}
          >
            Check again
          </Button>
        )}
      </div>

      {info?.available && (
        <div className="space-y-3 rounded-xl border border-accent bg-accent-soft p-4">
          <p className="font-medium text-ink">Version {info.available} is available.</p>
          {info.notes && (
            <div className="space-y-2 text-sm text-muted [&_strong]:text-ink">
              <Markdown>{info.notes}</Markdown>
            </div>
          )}
          {installing ? (
            <div className="space-y-2">
              <div className="h-2 overflow-hidden rounded-full bg-surface">
                <div className="h-full rounded-full bg-accent transition-all" style={{ width: `${percent ?? 5}%` }} />
              </div>
              <p className="text-sm text-muted">
                {percent === 100
                  ? "Installing. TuxBridge will close and open again by itself in a moment."
                  : progress
                    ? `Downloading… ${mb(progress[0])}${progress[1] ? ` of ${mb(progress[1])}` : ""} MB`
                    : "Starting the download…"}
              </p>
            </div>
          ) : (
            <>
              <Button variant="primary" icon={<Download className="size-4" />} onClick={install}>
                Update TuxBridge
              </Button>
              <p className="text-xs text-muted">
                TuxBridge closes while it updates and opens again afterwards. Your systems, desktops and settings stay as they
                are; save any work in an open desktop window first.
              </p>
            </>
          )}
        </div>
      )}

      {error && <Notice tone="bad">{error}</Notice>}
    </Card>
  );
}
