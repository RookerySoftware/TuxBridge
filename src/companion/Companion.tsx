// Pip, the companion: a penguin in the corner of the window that gives tips for each page,
// explains the buttons you point at, shares fun facts and suggestions, walks you through the app on
// first launch and through a new system when you asked it to "Show me around". In Kids mode it
// teaches, with little lessons and a quiz. Off by default, except in Kids mode and during tours.

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import Markdown, { type Components } from "react-markdown";
import { BookOpen, ChevronLeft, ChevronRight, Minus, Shuffle, Star, X } from "lucide-react";
import { api, type Distro } from "../api";
import { useCompanionOn, useKidsMode } from "../prefs";
import Penguin, { type Mood } from "./Penguin";
import {
  appTour, explanations, funFacts, kidsFacts, kidsLessons, pageTips, systemTour,
  type Action, type Bubble, type KidsLesson, type Page, type TourStep,
} from "./content";
import {
  addStar, finishShowAround, markPageSeen, onStartAppTour, pendingShowArounds, pointing, seenPages, setTourDone, stars, tourDone,
} from "./store";

const md: Components = {
  p: ({ children }) => <p className="leading-relaxed">{children}</p>,
  strong: ({ children }) => <strong className="font-semibold">{children}</strong>,
  code: ({ children }) => <code className="rounded bg-surface-2 px-1 py-0.5 font-mono text-[12px]">{children}</code>,
  ul: ({ children }) => <ul className="list-disc space-y-0.5 pl-4">{children}</ul>,
};

const pick = <T,>(list: T[]) => list[Math.floor(Math.random() * list.length)];

/** What a hovered element is called: its aria-label, title, or text. */
function labelOf(el: Element): string {
  const own = el.getAttribute("aria-label") || el.getAttribute("placeholder") || el.getAttribute("title");
  if (own) return own.trim();
  return (el.textContent ?? "").trim().split("\n")[0].trim();
}

function explain(label: string): string | null {
  const l = label.toLowerCase();
  if (!l) return null;
  // The longest key that the label starts with, so "Restart WSL" beats "Restart".
  let best: [string, string] | null = null;
  for (const e of explanations) {
    if (l.startsWith(e[0].toLowerCase()) && (!best || e[0].length > best[0].length)) best = e;
  }
  return best?.[1] ?? null;
}

type Tour = { kind: "app"; steps: TourStep[] } | { kind: "system"; name: string; steps: TourStep[] };

export default function Companion({ page, distros, onGo, onHelp }: {
  page: Page;
  distros: Distro[] | null;
  onGo: (page: Page) => void;
  onHelp: (topic: string) => void;
}) {
  const [kids] = useKidsMode();
  const [on, setOn] = useCompanionOn();
  const [tour, setTour] = useState<Tour | null>(null);
  const [step, setStep] = useState(0);
  const [open, setOpen] = useState(false);
  const [bubble, setBubble] = useState<Bubble | null>(null);
  const [hint, setHint] = useState(false);
  const [lessons, setLessons] = useState(false);
  const visible = on || kids || !!tour;

  const say = useCallback((b: Bubble, show = true) => {
    setBubble(b);
    setLessons(false);
    if (show) setOpen(true);
    else setHint(true);
  }, []);

  // ---- Tours ----

  const startApp = useCallback(() => {
    setTour({ kind: "app", steps: appTour });
    setStep(0);
    setOpen(true);
  }, []);
  useEffect(() => {
    const t = setTimeout(() => !tourDone() && startApp(), 900);
    const off = onStartAppTour(startApp);
    return () => {
      clearTimeout(t);
      off();
    };
  }, [startApp]);

  // A new system the person asked to be shown around, once it is ready (and not during another tour).
  useEffect(() => {
    if (tour || !distros || !tourDone()) return;
    for (const p of pendingShowArounds()) {
      const d = distros.find((x) => x.name === p.name);
      if (!d || d.busy) continue;
      const desktop = d.info.desktop ? d.catalog?.desktop?.name ?? d.info.desktop : null;
      setTour({ kind: "system", name: d.name, steps: systemTour(d.name, p.catalogId, d.catalog?.title ?? d.name, desktop, d.info.hyprland) });
      setStep(0);
      setOpen(true);
      return;
    }
  }, [distros, tour]);

  const endTour = (keep?: boolean) => {
    if (tour?.kind === "app") {
      setTourDone();
      if (keep !== undefined && !kids) setOn(keep);
    }
    if (tour?.kind === "system") finishShowAround(tour.name);
    setTour(null);
    setOpen(false);
  };

  const current = tour?.steps[step];
  useEffect(() => {
    if (current?.page && current.page !== page) onGo(current.page);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [current]);

  // ---- Page tips ----

  useEffect(() => {
    if (!visible || tour) return;
    const tips = pageTips[page][kids ? "kids" : "normal"];
    const first = !seenPages().includes(page);
    markPageSeen(page);
    say({ title: kids ? "Pip says" : "Tip", text: first ? tips[0] : pick(tips) }, first);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [page, visible, kids]);

  // A dialog opening gets the room: the bubble steps aside (pointing at things still works).
  useEffect(() => {
    if (!visible || tour) return;
    let had = false;
    const t = setInterval(() => {
      const has = !!document.querySelector("[role=dialog]");
      if (has && !had) setOpen(false);
      had = has;
    }, 300);
    return () => clearInterval(t);
  }, [visible, tour]);

  // ---- Pointing at buttons ----

  const hovered = useRef<{ timer?: number; fromHover: boolean; closeTimer?: number }>({ fromHover: false });
  useEffect(() => {
    if (!visible || tour) return;
    const h = hovered.current;
    const over = (e: MouseEvent) => {
      if (!pointing()) return;
      const target = e.target as Element | null;
      if (!target || target.closest("[data-companion]")) return;
      const el = target.closest("button, a, input, select, [role=radio], [role=switch]");
      clearTimeout(h.timer);
      if (!el) return;
      const label = labelOf(el);
      const text = explain(label);
      if (!text) return;
      h.timer = window.setTimeout(() => {
        clearTimeout(h.closeTimer);
        h.fromHover = true;
        setBubble({ title: label.length > 40 ? undefined : label, text, mood: "think" });
        setLessons(false);
        setOpen(true);
      }, 700);
    };
    const out = (e: MouseEvent) => {
      const target = e.target as Element | null;
      if (!target?.closest("button, a, input, select, [role=radio], [role=switch]")) return;
      clearTimeout(h.timer);
      if (h.fromHover) {
        clearTimeout(h.closeTimer);
        h.closeTimer = window.setTimeout(() => {
          h.fromHover = false;
          setOpen(false);
        }, 4000);
      }
    };
    document.addEventListener("mouseover", over);
    document.addEventListener("mouseout", out);
    return () => {
      document.removeEventListener("mouseover", over);
      document.removeEventListener("mouseout", out);
      clearTimeout(h.timer);
      clearTimeout(h.closeTimer);
    };
  }, [visible, tour]);

  // ---- Something else to say ----

  const suggestion = useMemo((): Bubble | null => {
    if (!distros) return null;
    if (distros.length === 0) {
      return { title: "Suggestion", text: kids ? "There's no Linux here yet! Ask a grown-up to help you get one." : "You don't have a Linux system yet. Shall we pick one?", actions: [{ label: "Get Linux", go: "get" }] };
    }
    const bare = distros.find((d) => d.catalog?.desktop && !d.info.desktop && !d.info.hyprland && !d.busy);
    if (bare && !kids) {
      return { title: "Suggestion", text: `**${bare.name}** has no desktop yet. Want windows, a menu and a taskbar? Choose **Add the ${bare.catalog!.desktop!.name} desktop** in its **⋯** menu.`, actions: [{ label: "My systems", go: "systems" }] };
    }
    return { title: "Suggestion", text: kids ? "Want an adventure? Try the **Treasure Hunt** on the Learn page!" : "Try a **lesson** on the Learn page: it walks you through the terminal step by step.", actions: [{ label: "Go to Learn", go: "learn" }] };
  }, [distros, kids]);

  const another = () => {
    const roll = Math.random();
    if (roll < 0.25 && suggestion) say(suggestion);
    else if (roll < 0.55) say({ title: kids ? "Pip says" : "Tip", text: pick(pageTips[page][kids ? "kids" : "normal"]) });
    else say({ title: "Fun fact", text: pick(kids ? [...kidsFacts, ...kidsFacts, ...funFacts.slice(0, 3)] : funFacts), mood: "wow" });
  };

  const act = (a: Action) => {
    if (a.go) onGo(a.go);
    if (a.help) onHelp(a.help);
    if (a.run && tour?.kind === "system") {
      (a.run === "terminal" ? api.openTerminal(tour.name) : api.openFiles(tour.name)).catch(() => {});
    }
  };

  if (!visible) return null;

  // ---- Render ----

  const mood: Mood = current?.mood ?? bubble?.mood ?? "idle";
  const last = tour ? step === tour.steps.length - 1 : false;

  return (
    <>
      {tour && current?.target && <Spotlight target={current.target} />}
      <div data-companion className="pointer-events-none fixed bottom-4 right-4 z-50 flex flex-col items-end gap-2">
        {open && (tour || bubble || lessons) && (
          <div className="pointer-events-auto w-80 rounded-2xl border border-line bg-surface p-4 text-sm shadow-xl animate-in">
            <div className="mb-2 flex items-center gap-1">
              <span className="flex-1 font-display font-semibold">
                {tour ? current?.title : lessons ? "Pip's lessons" : bubble?.title ?? "Pip"}
              </span>
              {!tour && kids && (
                <IconBtn label="Pip's lessons" onClick={() => setLessons(!lessons)}>
                  <BookOpen className="size-4" />
                </IconBtn>
              )}
              {!tour && (
                <IconBtn label="Something else" onClick={another}>
                  <Shuffle className="size-4" />
                </IconBtn>
              )}
              {!tour && (
                <IconBtn label="Hide" onClick={() => setOpen(false)}>
                  <Minus className="size-4" />
                </IconBtn>
              )}
              {tour ? (
                <IconBtn label="Skip the tour" onClick={() => endTour()}>
                  <X className="size-4" />
                </IconBtn>
              ) : (
                !kids && (
                  <IconBtn label="Turn Pip off (Settings brings it back)" onClick={() => setOn(false)}>
                    <X className="size-4" />
                  </IconBtn>
                )
              )}
            </div>

            {tour && current ? (
              <div className="space-y-3">
                <div className="space-y-2 text-ink">
                  <Markdown components={md}>{current.text}</Markdown>
                </div>
                <Actions actions={current.actions} onAct={act} />
                <div className="flex items-center justify-between gap-2 pt-1">
                  <span className="text-xs text-muted">
                    {step + 1} of {tour.steps.length}
                  </span>
                  <div className="flex gap-1.5">
                    {step === 0 && tour.kind === "app" && (
                      <SmallBtn onClick={() => endTour()}>Skip</SmallBtn>
                    )}
                    {step > 0 && (
                      <SmallBtn onClick={() => setStep(step - 1)}>
                        <ChevronLeft className="size-3.5" /> Back
                      </SmallBtn>
                    )}
                    {!last && (
                      <SmallBtn primary onClick={() => setStep(step + 1)}>
                        {step === 0 ? "Show me" : "Next"} <ChevronRight className="size-3.5" />
                      </SmallBtn>
                    )}
                    {last && tour.kind === "app" && !kids && (
                      <>
                        <SmallBtn onClick={() => endTour(false)}>No thanks</SmallBtn>
                        <SmallBtn primary onClick={() => endTour(true)}>Stay, Pip!</SmallBtn>
                      </>
                    )}
                    {last && (tour.kind === "system" || kids) && (
                      <SmallBtn primary onClick={() => endTour()}>Done</SmallBtn>
                    )}
                  </div>
                </div>
              </div>
            ) : lessons ? (
              <KidsLessons onStar={() => say({ title: "Yay!", text: "Great learning! Want another lesson? Press the **book**.", mood: "happy" })} />
            ) : bubble ? (
              <div className="space-y-3">
                <div className="space-y-2 text-ink">
                  <Markdown components={md}>{bubble.text}</Markdown>
                </div>
                <Actions actions={bubble.actions} onAct={act} />
              </div>
            ) : null}
          </div>
        )}
        <button
          className="pointer-events-auto relative rounded-full transition hover:scale-105 active:scale-95 pip-bob"
          aria-label={open ? "Hide Pip's bubble" : "Talk to Pip"}
          onClick={() => {
            setHint(false);
            hovered.current.fromHover = false;
            if (open) setOpen(false);
            else if (!bubble && !tour) another();
            else setOpen(true);
          }}
        >
          <Penguin mood={mood} kids={kids} size={kids ? 72 : 60} className="drop-shadow-lg" />
          {hint && !open && <span className="absolute right-1 top-1 size-3 rounded-full border-2 border-surface bg-accent" />}
        </button>
      </div>
    </>
  );
}

function IconBtn({ label, onClick, children }: { label: string; onClick: () => void; children: React.ReactNode }) {
  return (
    <button aria-label={label} title={label} onClick={onClick} className="rounded-md p-1 text-muted transition hover:bg-surface-2 hover:text-ink">
      {children}
    </button>
  );
}

function SmallBtn({ primary, onClick, children }: { primary?: boolean; onClick: () => void; children: React.ReactNode }) {
  return (
    <button
      onClick={onClick}
      className={`inline-flex h-7 items-center gap-1 rounded-md px-2.5 text-xs font-medium transition ${primary ? "bg-accent text-accent-ink hover:opacity-90" : "border border-line hover:bg-surface-2"}`}
    >
      {children}
    </button>
  );
}

function Actions({ actions, onAct }: { actions?: Action[]; onAct: (a: Action) => void }) {
  if (!actions?.length) return null;
  return (
    <div className="flex flex-wrap gap-1.5">
      {actions.map((a) => (
        <SmallBtn key={a.label} onClick={() => onAct(a)}>
          {a.label}
        </SmallBtn>
      ))}
    </div>
  );
}

/** Dims the window except for the element being talked about. */
function Spotlight({ target }: { target: string }) {
  const [rect, setRect] = useState<DOMRect | null>(null);
  useEffect(() => {
    const find = () => {
      const el = document.querySelector(`[data-tour="${CSS.escape(target)}"]`);
      setRect(el ? el.getBoundingClientRect() : null);
    };
    find();
    const t = setInterval(find, 250);
    return () => clearInterval(t);
  }, [target]);
  if (!rect) return null;
  const pad = 6;
  return (
    <div
      className="pointer-events-none fixed z-40 rounded-xl ring-2 ring-accent transition-all duration-300"
      style={{
        left: rect.left - pad,
        top: rect.top - pad,
        width: rect.width + pad * 2,
        height: rect.height + pad * 2,
        boxShadow: "0 0 0 9999px rgb(0 0 0 / 0.55)",
      }}
    />
  );
}

function KidsLessons({ onStar }: { onStar: () => void }) {
  const [done, setDone] = useState(stars);
  const [lesson, setLesson] = useState<KidsLesson | null>(null);
  const [card, setCard] = useState(0);
  const [picked, setPicked] = useState<number | null>(null);

  if (!lesson) {
    return (
      <div className="space-y-2">
        <p className="text-muted">
          You have <b className="text-ink">{done.length}</b> of {kidsLessons.length} stars. Pick a lesson!
        </p>
        <div className="max-h-64 space-y-1 overflow-y-auto pr-1">
          {kidsLessons.map((l) => (
            <button
              key={l.id}
              onClick={() => {
                setLesson(l);
                setCard(0);
                setPicked(null);
              }}
              className="flex w-full items-center gap-2 rounded-lg px-2 py-1.5 text-left transition hover:bg-surface-2"
            >
              <Star className={`size-4 shrink-0 ${done.includes(l.id) ? "fill-warn text-warn" : "text-muted"}`} />
              {l.title}
            </button>
          ))}
        </div>
      </div>
    );
  }

  const quiz = card >= lesson.cards.length;
  const right = picked === lesson.quiz.answer;
  return (
    <div className="space-y-3">
      <div className="flex items-center gap-2">
        <span className="flex-1 font-medium">{lesson.title}</span>
        <span className="text-xs text-muted">
          {Math.min(card + 1, lesson.cards.length + 1)} / {lesson.cards.length + 1}
        </span>
      </div>
      {!quiz ? (
        <div className="space-y-2">
          <Markdown components={md}>{lesson.cards[card]}</Markdown>
        </div>
      ) : (
        <div className="space-y-2">
          <p className="font-medium">Quiz time! {lesson.quiz.question}</p>
          {lesson.quiz.options.map((o, i) => (
            <button
              key={o}
              disabled={right}
              onClick={() => {
                setPicked(i);
                if (i === lesson.quiz.answer && !done.includes(lesson.id)) {
                  addStar(lesson.id);
                  setDone(stars());
                }
              }}
              className={`block w-full rounded-lg border px-3 py-1.5 text-left transition ${
                picked === i ? (right ? "border-good bg-good-soft text-good" : "border-bad bg-bad-soft text-bad") : "border-line hover:bg-surface-2"
              }`}
            >
              {o}
            </button>
          ))}
          {picked !== null && (
            <p className={right ? "text-good" : "text-muted"}>{right ? `Yes! ${lesson.quiz.why} ⭐` : "Not quite. Try another one!"}</p>
          )}
        </div>
      )}
      <div className="flex justify-between">
        <SmallBtn onClick={() => (card === 0 ? setLesson(null) : setCard(card - 1))}>
          <ChevronLeft className="size-3.5" /> Back
        </SmallBtn>
        {!quiz ? (
          <SmallBtn primary onClick={() => setCard(card + 1)}>
            Next <ChevronRight className="size-3.5" />
          </SmallBtn>
        ) : (
          right && (
            <SmallBtn
              primary
              onClick={() => {
                onStar();
                setLesson(null);
              }}
            >
              More lessons
            </SmallBtn>
          )
        )}
      </div>
    </div>
  );
}
