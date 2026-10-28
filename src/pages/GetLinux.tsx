// The catalogue: Linux systems described for newcomers, and the short form that installs one.

import { useEffect, useState } from "react";
import { Check, ChevronDown, Clock, ExternalLink, HardDrive, LayoutDashboard, Monitor, Sparkles, SquareTerminal } from "lucide-react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { api, inApp, type CatalogEntry, type DesktopChoice, type InstallRequest } from "../api";
import { Badge, Button, Card, Dialog, Field, Monogram, Notice, Toggle, inputClass } from "../components/ui";
import { requestShowAround } from "../companion/store";

/** The desktop a new system starts with: its own by default, Hyprland instead, or none. */
function defaultChoice(entry: CatalogEntry): DesktopChoice {
  if (entry.hyprland === "included") return "hyprland";
  if (entry.desktop) return "own";
  return entry.recommended && entry.hyprland === "optional" ? "hyprland" : "none";
}

function ChoiceCard({ selected, onSelect, icon, title, text, disabled }: {
  selected: boolean; onSelect: () => void; icon: React.ReactNode; title: string; text: string; disabled?: boolean;
}) {
  return (
    <button
      type="button"
      role="radio"
      aria-checked={selected}
      disabled={disabled}
      onClick={onSelect}
      className={`relative flex flex-col items-start gap-1 rounded-xl border p-3.5 text-left transition
        focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-50
        ${selected ? "border-accent bg-accent-soft" : "border-line hover:border-accent/50"}`}
    >
      {selected && <Check className="absolute right-3 top-3 size-4 text-accent" />}
      <span className="flex items-center gap-2 text-sm font-medium">
        {icon}
        {title}
      </span>
      <span className="text-xs text-muted">{text}</span>
    </button>
  );
}

function InstallDialog({ entry, onClose, onStarted }: { entry: CatalogEntry; onClose: () => void; onStarted: () => void }) {
  const omarchy = entry.source.kind === "omarchy";
  const [name, setName] = useState("");
  const [user, setUser] = useState("");
  const [password, setPassword] = useState("");
  const [again, setAgain] = useState("");
  const [choice, setChoice] = useState<DesktopChoice>(defaultChoice(entry));
  const hyprland = choice === "hyprland";
  const [windowsKey, setWindowsKey] = useState(true);
  const [thirdParty, setThirdParty] = useState(false);
  const [setDefault, setSetDefault] = useState(false);
  const [showAround, setShowAround] = useState(true);
  const [advanced, setAdvanced] = useState(false);
  const [location, setLocation] = useState("");
  const [locationTouched, setLocationTouched] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    api.suggestName(entry.id).then(setName);
    api.suggestUser().then(setUser);
  }, [entry.id]);
  useEffect(() => {
    if (!locationTouched && name) api.defaultLocation(name).then(setLocation);
  }, [name, locationTouched]);

  const mismatch = !omarchy && again.length > 0 && password !== again;
  const request = (): InstallRequest => ({
    catalogId: entry.id,
    name,
    user: omarchy ? null : user,
    password: omarchy ? null : password,
    desktop: omarchy ? "hyprland" : choice,
    mainKey: windowsKey ? "SUPER" : "ALT",
    allowThirdParty: thirdParty,
    location: locationTouched ? location : null,
    setDefault,
  });

  async function install() {
    if (!omarchy && password !== again) {
      setError("The two passwords are not the same.");
      return;
    }
    setBusy(true);
    setError(null);
    try {
      await api.startInstall(request());
      if (showAround) requestShowAround(name, entry.id);
      onStarted();
    } catch (e) {
      setError(String(e));
      setBusy(false);
    }
  }

  return (
    <Dialog title={`Install ${entry.title}`} onClose={onClose} wide>
      <form
        className="space-y-5"
        onSubmit={(e) => {
          e.preventDefault();
          install();
        }}
      >
        <Field label="Name" hint="How this system is called on your PC. You can have several of the same kind.">
          <input className={inputClass} value={name} onChange={(e) => setName(e.target.value)} spellCheck={false} required />
        </Field>

        {omarchy ? (
          <Notice icon={<Sparkles className="size-4 text-accent" />}>
            Omarchy asks for your account, keyboard and time zone itself, the first time you open it, just like a new PC.
          </Notice>
        ) : (
          <div className="grid gap-4 sm:grid-cols-3">
            <Field label="User name" hint="Lower-case letters and digits.">
              <input className={inputClass} value={user} onChange={(e) => setUser(e.target.value)} spellCheck={false} required />
            </Field>
            <Field label="Password" hint="Linux asks for it when you install software.">
              <input className={inputClass} type="password" value={password} onChange={(e) => setPassword(e.target.value)} required />
            </Field>
            <Field label="Password again" error={mismatch ? "Not the same as above." : null}>
              <input className={inputClass} type="password" value={again} onChange={(e) => setAgain(e.target.value)} required />
            </Field>
          </div>
        )}

        {omarchy ? (
          <Notice icon={<LayoutDashboard className="size-4 text-accent" />}>Omarchy is a Hyprland desktop; it comes with it.</Notice>
        ) : (
          <div className="space-y-3">
            <span className="text-sm font-medium">Desktop</span>
            <div role="radiogroup" aria-label="Desktop" className="grid gap-2 sm:grid-cols-3">
              {entry.desktop && (
                <ChoiceCard
                  selected={choice === "own"}
                  onSelect={() => setChoice("own")}
                  icon={<Monitor className="size-4 text-accent" />}
                  title={entry.desktop.name}
                  text={`${entry.title}'s own desktop. Familiar, with a menu, taskbar and windows.`}
                />
              )}
              {entry.hyprland !== "no" && (
                <ChoiceCard
                  selected={choice === "hyprland"}
                  onSelect={() => setChoice("hyprland")}
                  icon={<LayoutDashboard className="size-4 text-accent" />}
                  title="Hyprland"
                  text="A modern desktop where windows arrange themselves in tiles, driven by the keyboard."
                />
              )}
              <ChoiceCard
                selected={choice === "none"}
                onSelect={() => setChoice("none")}
                icon={<SquareTerminal className="size-4 text-accent" />}
                title="None"
                text="Just a terminal. Linux apps you install still open in their own windows."
              />
            </div>
            {choice === "own" && entry.desktop?.note && <p className="text-xs text-muted">{entry.desktop.note}</p>}
            {choice === "hyprland" && entry.hyprlandNote && <p className="text-xs text-muted">{entry.hyprlandNote}</p>}
            {hyprland && (
              <Toggle
                checked={windowsKey}
                onChange={setWindowsKey}
                label="Use the Windows key for the desktop's shortcuts"
                description="Recommended. Off: the Alt key instead."
              />
            )}
            {hyprland && entry.thirdParty && (
              <Toggle checked={thirdParty} onChange={setThirdParty} label="Allow the community package sources" description={entry.thirdParty} />
            )}
            <p className="text-xs text-muted">You can add another desktop later.</p>
          </div>
        )}

        <Toggle
          checked={showAround}
          onChange={setShowAround}
          label="Show me around"
          description="When it is ready, Pip the penguin walks you through your new system: opening it, installing software, updates and where to learn more."
        />

        <div>
          <button type="button" className="inline-flex items-center gap-1 text-sm text-muted hover:text-ink" onClick={() => setAdvanced(!advanced)}>
            <ChevronDown className={`size-4 transition ${advanced ? "rotate-180" : ""}`} /> More options
          </button>
          {advanced && (
            <div className="mt-3 space-y-4 animate-in">
              <Field label="Folder for its disk" hint="Needs a few GB free. A fast drive (SSD) makes Linux much quicker.">
                <input
                  className={inputClass}
                  value={location}
                  onChange={(e) => {
                    setLocation(e.target.value);
                    setLocationTouched(true);
                  }}
                  spellCheck={false}
                />
              </Field>
              <Toggle
                checked={setDefault}
                onChange={setSetDefault}
                label="Make it the default system"
                description="The one that opens when you type wsl in a terminal."
              />
            </div>
          )}
        </div>

        {error && <Notice tone="bad">{error}</Notice>}

        <div className="flex items-center justify-between gap-3 border-t border-line pt-4">
          <span className="text-xs text-muted">
            About {entry.minutes} minutes{choice === "own" && entry.desktop ? ` plus ${entry.desktop.minutes} for the desktop` : ""},{" "}
            {(entry.sizeGb + (choice === "own" && entry.desktop ? entry.desktop.sizeGb : 0)).toFixed(1)} GB.
          </span>
          <div className="flex gap-2">
            <Button type="button" onClick={onClose}>
              Cancel
            </Button>
            <Button type="submit" variant="primary" busy={busy} disabled={mismatch || (hyprland && !!entry.thirdParty && !thirdParty)}>
              Install
            </Button>
          </div>
        </div>
      </form>
    </Dialog>
  );
}

export default function GetLinux({ onStarted }: { onStarted: () => void }) {
  const [catalog, setCatalog] = useState<CatalogEntry[]>([]);
  const [chosen, setChosen] = useState<CatalogEntry | null>(null);
  useEffect(() => {
    api.catalog().then(setCatalog);
  }, []);

  return (
    <div className="space-y-6">
      <header>
        <h1 className="font-display text-2xl font-semibold">Get Linux</h1>
        <p className="text-muted">
          Each one is a complete Linux system that runs next to Windows. Not sure? <b className="text-ink">Ubuntu</b> is the most common
          choice; <b className="text-ink">Arch Linux</b> is the best with the Hyprland desktop.
        </p>
      </header>
      <div className="grid gap-4 lg:grid-cols-2">
        {catalog.map((e) => (
          <Card key={e.id} className="flex flex-col p-5 transition hover:border-accent/50">
            <div className="flex items-start gap-4">
              <Monogram title={e.title} accent={e.accent} size="lg" />
              <div className="min-w-0 flex-1">
                <div className="flex flex-wrap items-center gap-2">
                  <h3 className="font-display text-base font-semibold">{e.title}</h3>
                  {e.recommended && <Badge tone="accent">Recommended</Badge>}
                </div>
                <p className="text-sm font-medium">{e.tagline}</p>
              </div>
            </div>
            <p className="mt-3 flex-1 text-sm leading-relaxed text-muted">{e.description}</p>
            <div className="mt-3 flex flex-wrap gap-1.5">
              {e.desktop && <Badge tone="accent">{e.desktop.name} desktop</Badge>}
              {e.hyprland === "included" && <Badge tone="accent">Hyprland desktop</Badge>}
              {e.hyprland === "optional" && <Badge>Hyprland available</Badge>}
              <Badge>
                <Clock className="size-3" /> {e.minutes} min
              </Badge>
              <Badge>
                <HardDrive className="size-3" /> {e.sizeGb} GB
              </Badge>
            </div>
            <div className="mt-4 flex items-center justify-between">
              <button
                className="inline-flex items-center gap-1 text-xs text-muted hover:text-ink"
                onClick={() => (inApp ? openUrl(e.website) : window.open(e.website))}
              >
                Website <ExternalLink className="size-3" />
              </button>
              <Button variant="primary" onClick={() => setChosen(e)}>
                Install
              </Button>
            </div>
          </Card>
        ))}
      </div>
      {chosen && (
        <InstallDialog
          entry={chosen}
          onClose={() => setChosen(null)}
          onStarted={() => {
            setChosen(null);
            onStarted();
          }}
        />
      )}
    </div>
  );
}
