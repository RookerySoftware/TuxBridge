// Shown instead of the app until this PC can run WSL 2: one explanation and one next step.

import { useState } from "react";
import { AlertTriangle, Cpu, Download, RefreshCw, RotateCcw, ShieldCheck } from "lucide-react";
import { api, type Readiness } from "../api";
import { Button, Card, Notice } from "../components/ui";

interface Screen {
  icon: React.ReactNode;
  title: string;
  body: React.ReactNode;
  action?: { label: string; fix?: string };
}

function screenFor(r: Readiness): Screen {
  switch (r.state) {
    case "windowsTooOld":
      return {
        icon: <AlertTriangle className="size-6" />,
        title: "Windows needs an update first",
        body: (
          <p>
            Linux on Windows needs Windows 10 version 21H2 or newer, and this PC has an older version (build {r.build}). Open{" "}
            <b>Settings → Windows Update</b>, install what it offers, restart, and open EasyWSL again.
          </p>
        ),
        action: { label: "Check again" },
      };
    case "needsWindowsUpdate":
      return {
        icon: <AlertTriangle className="size-6" />,
        title: "Windows is missing a few updates",
        body: (
          <p>
            This Windows 10 needs its updates from November 2022 or later to run Linux. Open <b>Settings → Update & Security →
            Windows Update</b>, install everything it offers, restart, and open EasyWSL again.
          </p>
        ),
        action: { label: "Check again" },
      };
    case "virtualizationOff":
      return r.virtualMachine
        ? {
            icon: <Cpu className="size-6" />,
            title: "This virtual machine cannot run Linux inside",
            body: (
              <p>
                Windows is running in a virtual machine ({r.virtualMachine}) that does not allow virtual machines inside it, which
                Linux on Windows needs. Shut the virtual machine down and turn on <b>nested virtualization</b> in its settings
                (VirtualBox: <i>Settings → System → Processor → Enable Nested VT-x/AMD-V</i>; VMware: <i>Virtualize Intel VT-x/EPT</i>).
              </p>
            ),
            action: { label: "Check again" },
          }
        : {
            icon: <Cpu className="size-6" />,
            title: "Virtualization is switched off",
            body: (
              <>
                <p>
                  Linux runs in a lightweight virtual machine, and your PC's processor has that feature switched off in its firmware
                  (the BIOS or UEFI settings, which you reach while the PC starts).
                </p>
                <p>
                  Restart the PC, open those settings (often with <b>Del</b>, <b>F2</b> or <b>F10</b> right after switching on), and turn
                  on <b>Intel VT-x</b> or <b>AMD SVM</b>, usually under CPU or Advanced. Your PC maker's support page shows the exact
                  steps.
                </p>
              </>
            ),
            action: { label: "Check again" },
          };
    case "wslMissing":
      return {
        icon: <Download className="size-6" />,
        title: "Let's add Linux support to Windows",
        body: (
          <>
            <p>
              Windows runs Linux through a part called WSL (Windows Subsystem for Linux). It comes from Microsoft and is free. EasyWSL
              downloads and installs it for you.
            </p>
            <p>Windows will ask for permission: choose <b>Yes</b>.</p>
          </>
        ),
        action: { label: "Install WSL", fix: "wslMissing" },
      };
    case "featuresOff":
      return {
        icon: <ShieldCheck className="size-6" />,
        title: "Switch on Linux support",
        body: (
          <>
            <p>
              WSL is installed, but two parts of Windows it needs are switched off. EasyWSL switches them on; after that, Windows needs
              one restart.
            </p>
            <p>Windows will ask for permission: choose <b>Yes</b>.</p>
          </>
        ),
        action: { label: "Switch on", fix: "featuresOff" },
      };
    case "restartNeeded":
      return {
        icon: <RotateCcw className="size-6" />,
        title: "Restart your PC to finish",
        body: <p>Windows needs a restart to finish setting up Linux support. Save your work, restart, and open EasyWSL again.</p>,
      };
    case "ready":
      return { icon: null, title: "", body: null };
  }
}

export default function Setup({ readiness, onChange }: { readiness: Readiness; onChange: (r: Readiness) => void }) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const screen = screenFor(readiness);

  async function act() {
    setBusy(true);
    setError(null);
    try {
      onChange(screen.action?.fix ? await api.systemFix(screen.action.fix) : await api.systemCheck());
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="flex h-full items-center justify-center p-8">
      <Card className="w-full max-w-lg p-8 animate-in">
        <div className="mb-5 flex size-12 items-center justify-center rounded-2xl bg-accent-soft text-accent">{screen.icon}</div>
        <h1 className="font-display text-2xl font-semibold">{screen.title}</h1>
        <div className="mt-3 space-y-3 leading-relaxed text-muted [&_b]:text-ink">{screen.body}</div>
        {error && (
          <div className="mt-4">
            <Notice tone="bad" icon={<AlertTriangle className="size-4 text-bad" />}>
              {error}
            </Notice>
          </div>
        )}
        {screen.action && (
          <div className="mt-6 flex gap-2">
            <Button variant="primary" busy={busy} onClick={act} icon={screen.action.fix ? undefined : <RefreshCw className="size-4" />}>
              {screen.action.label}
            </Button>
          </div>
        )}
      </Card>
    </div>
  );
}
