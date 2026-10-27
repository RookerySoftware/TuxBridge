// Small building blocks used across the pages.

import { type ButtonHTMLAttributes, type ReactNode, useEffect, useRef, useState } from "react";
import { Loader2, MoreHorizontal, X } from "lucide-react";

type Variant = "primary" | "secondary" | "ghost" | "danger";

const variants: Record<Variant, string> = {
  primary: "bg-accent text-accent-ink hover:brightness-110 shadow-sm",
  secondary: "bg-surface text-ink border border-line hover:bg-surface-2",
  ghost: "text-ink hover:bg-surface-2",
  danger: "bg-bad text-white hover:brightness-110",
};

export function Button({
  variant = "secondary",
  icon,
  busy,
  className = "",
  children,
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & { variant?: Variant; icon?: ReactNode; busy?: boolean }) {
  return (
    <button
      {...props}
      disabled={props.disabled || busy}
      className={`inline-flex items-center justify-center gap-2 rounded-lg px-3.5 h-9 text-sm font-medium transition
        focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent
        disabled:opacity-50 disabled:pointer-events-none ${variants[variant]} ${className}`}
    >
      {busy ? <Loader2 className="size-4 animate-spin" /> : icon}
      {children}
    </button>
  );
}

export function IconButton({ label, children, ...props }: ButtonHTMLAttributes<HTMLButtonElement> & { label: string }) {
  return (
    <button
      {...props}
      title={label}
      aria-label={label}
      className="inline-flex size-9 items-center justify-center rounded-lg text-muted transition hover:bg-surface-2 hover:text-ink
        focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-40 disabled:pointer-events-none"
    >
      {children}
    </button>
  );
}

export function Card({ className = "", children }: { className?: string; children: ReactNode }) {
  return <div className={`rounded-xl border border-line bg-surface shadow-[var(--shadow)] ${className}`}>{children}</div>;
}

const badgeTones = {
  neutral: "bg-surface-2 text-muted",
  good: "bg-good-soft text-good",
  warn: "bg-warn-soft text-warn",
  bad: "bg-bad-soft text-bad",
  accent: "bg-accent-soft text-accent",
};

export function Badge({ tone = "neutral", children }: { tone?: keyof typeof badgeTones; children: ReactNode }) {
  return (
    <span className={`inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs font-medium ${badgeTones[tone]}`}>
      {children}
    </span>
  );
}

/** A distro's mark: its initial on its colour. */
export function Monogram({ title, accent, size = "md" }: { title: string; accent: string; size?: "md" | "lg" }) {
  const dims = size === "lg" ? "size-14 text-2xl rounded-2xl" : "size-11 text-lg rounded-xl";
  return (
    <div
      className={`${dims} shrink-0 flex items-center justify-center font-display font-semibold text-white shadow-sm`}
      style={{ background: `linear-gradient(135deg, ${accent}, color-mix(in srgb, ${accent} 70%, black))` }}
      aria-hidden
    >
      {title.charAt(0).toUpperCase()}
    </div>
  );
}

export function Field({ label, hint, error, children }: { label: string; hint?: ReactNode; error?: string | null; children: ReactNode }) {
  return (
    <label className="block space-y-1.5">
      <span className="text-sm font-medium">{label}</span>
      {children}
      {error ? <span className="block text-xs text-bad">{error}</span> : hint ? <span className="block text-xs text-muted">{hint}</span> : null}
    </label>
  );
}

export const inputClass =
  "w-full h-9 rounded-lg border border-line bg-surface px-3 text-sm outline-none transition placeholder:text-muted focus:border-accent focus:ring-2 focus:ring-accent/25";

export function Toggle({ checked, onChange, label, description, disabled }: { checked: boolean; onChange: (v: boolean) => void; label: ReactNode; description?: ReactNode; disabled?: boolean }) {
  return (
    <label className={`flex items-start gap-3 ${disabled ? "opacity-50" : "cursor-pointer"}`}>
      <button
        type="button"
        role="switch"
        aria-checked={checked}
        disabled={disabled}
        onClick={() => onChange(!checked)}
        className={`mt-0.5 relative inline-flex h-5 w-9 shrink-0 rounded-full transition focus-visible:outline-2 focus-visible:outline-accent
          ${checked ? "bg-accent" : "bg-line"}`}
      >
        <span className={`absolute top-0.5 size-4 rounded-full bg-white shadow transition-all ${checked ? "left-[18px]" : "left-0.5"}`} />
      </button>
      <span className="space-y-0.5">
        <span className="block text-sm font-medium">{label}</span>
        {description && <span className="block text-xs text-muted">{description}</span>}
      </span>
    </label>
  );
}

export function Dialog({ title, onClose, children, wide }: { title: string; onClose: () => void; children: ReactNode; wide?: boolean }) {
  const ref = useRef<HTMLDivElement>(null);
  // The newest onClose, without re-running the effects below: callers pass a new function on
  // every render, and the app re-renders every few seconds (its system list refreshes).
  const close = useRef(onClose);
  close.current = onClose;
  // Focus the first field once, when the dialog opens; never again while the user types.
  useEffect(() => {
    (ref.current?.querySelector<HTMLElement>("input") ?? ref.current?.querySelector<HTMLElement>("button"))?.focus();
  }, []);
  useEffect(() => {
    const key = (e: KeyboardEvent) => e.key === "Escape" && close.current();
    window.addEventListener("keydown", key);
    return () => window.removeEventListener("keydown", key);
  }, []);
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-6 animate-in" onMouseDown={onClose}>
      <div
        ref={ref}
        role="dialog"
        aria-modal
        aria-label={title}
        onMouseDown={(e) => e.stopPropagation()}
        className={`w-full ${wide ? "max-w-2xl" : "max-w-md"} max-h-full overflow-y-auto rounded-2xl border border-line bg-surface shadow-2xl`}
      >
        <div className="flex items-center justify-between px-6 pt-5">
          <h2 className="font-display text-lg font-semibold">{title}</h2>
          <IconButton label="Close" onClick={onClose}>
            <X className="size-4" />
          </IconButton>
        </div>
        <div className="px-6 pb-6 pt-3">{children}</div>
      </div>
    </div>
  );
}

export function Notice({ tone = "neutral", icon, children }: { tone?: "neutral" | "warn" | "bad" | "good"; icon?: ReactNode; children: ReactNode }) {
  const tones = { neutral: "bg-surface-2 text-ink", warn: "bg-warn-soft text-ink", bad: "bg-bad-soft text-ink", good: "bg-good-soft text-ink" };
  return (
    <div className={`flex gap-2.5 rounded-lg px-3.5 py-3 text-sm ${tones[tone]}`}>
      {icon && <span className="mt-0.5 shrink-0">{icon}</span>}
      <div className="space-y-1">{children}</div>
    </div>
  );
}

/** A "More" button opening a small menu of actions. */
export function Menu({ items }: { items: { label: string; icon: React.ReactNode; onClick: () => void; danger?: boolean; hidden?: boolean }[] }) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!open) return;
    const close = (e: MouseEvent) => !ref.current?.contains(e.target as Node) && setOpen(false);
    window.addEventListener("mousedown", close);
    return () => window.removeEventListener("mousedown", close);
  }, [open]);
  return (
    <div className="relative" ref={ref}>
      <IconButton label="More" onClick={() => setOpen(!open)}>
        <MoreHorizontal className="size-4" />
      </IconButton>
      {open && (
        <div className="absolute right-0 z-20 mt-1 w-52 rounded-xl border border-line bg-surface p-1 shadow-xl animate-in">
          {items.filter((i) => !i.hidden).map((i) => (
            <button
              key={i.label}
              onClick={() => {
                setOpen(false);
                i.onClick();
              }}
              className={`flex w-full items-center gap-2.5 rounded-lg px-2.5 py-2 text-left text-sm hover:bg-surface-2 ${i.danger ? "text-bad" : ""}`}
            >
              {i.icon}
              {i.label}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
