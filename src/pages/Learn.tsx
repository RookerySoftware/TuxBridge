// The Learn page: guided lessons (step-by-step walk-throughs) and quests (HackTheBox-style
// challenge sets you solve in a real system). A Kids mode swaps in gentler projects. Progress is
// kept in the browser (localStorage); flags are checked here against their SHA-256.

import { useEffect, useState } from "react";
import Markdown, { type Components } from "react-markdown";
import remarkGfm from "remark-gfm";
import { ArrowLeft, BadgeCheck, Baby, Check, ChevronRight, CircleHelp, GraduationCap, Lightbulb, Lock, Play, Rocket, Swords, Terminal } from "lucide-react";
import { type Distro } from "../api";
import { api } from "../api";
import { Badge, Button, Card, Notice } from "../components/ui";
import { lessons, type Lesson } from "../learn/lessons";
import { quests, hashFlag, type Quest } from "../learn/quests";
import { useKidsMode } from "../prefs";

const lessonsKey = "easywsl.learn.lessons";
const questKey = (id: string) => `easywsl.learn.quest.${id}`;

function load<T>(key: string, fallback: T): T {
  try {
    const raw = localStorage.getItem(key);
    return raw ? (JSON.parse(raw) as T) : fallback;
  } catch {
    return fallback;
  }
}
function save(key: string, value: unknown) {
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch {
    /* private mode, ignore */
  }
}

const md: Components = {
  p: ({ children }) => <p className="leading-relaxed">{children}</p>,
  ul: ({ children }) => <ul className="list-disc space-y-1 pl-5 leading-relaxed">{children}</ul>,
  ol: ({ children }) => <ol className="list-decimal space-y-1 pl-5 leading-relaxed">{children}</ol>,
  strong: ({ children }) => <strong className="font-semibold">{children}</strong>,
  code: ({ children }) => <code className="rounded bg-surface-2 px-1 py-0.5 font-mono text-[13px]">{children}</code>,
};

function Code({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);
  return (
    <div className="group relative">
      <pre className="overflow-x-auto rounded-lg bg-surface-2 px-4 py-3 font-mono text-[13px] leading-relaxed">{text}</pre>
      <button
        onClick={() => navigator.clipboard.writeText(text).then(() => (setCopied(true), setTimeout(() => setCopied(false), 1200)))}
        className="absolute right-2 top-2 rounded-md border border-line bg-surface px-2 py-1 text-xs text-muted opacity-0 transition group-hover:opacity-100"
      >
        {copied ? "Copied" : "Copy"}
      </button>
    </div>
  );
}

// ---- Lessons ----

function LessonView({ lesson, onBack, onDone }: { lesson: Lesson; onBack: () => void; onDone: () => void }) {
  const [step, setStep] = useState(0);
  const last = lesson.steps.length - 1;
  const s = lesson.steps[step];
  return (
    <div className="max-w-2xl space-y-5">
      <button onClick={onBack} className="flex items-center gap-1.5 text-sm text-muted hover:text-ink">
        <ArrowLeft className="size-4" /> All lessons
      </button>
      <div>
        <h1 className="font-display text-2xl font-semibold">{lesson.title}</h1>
        <div className="mt-2 flex items-center gap-2 text-xs text-muted">
          <span>Step {step + 1} of {lesson.steps.length}</span>
          <div className="h-1.5 flex-1 overflow-hidden rounded-full bg-surface-2">
            <div className="h-full rounded-full bg-accent transition-all" style={{ width: `${((step + 1) / lesson.steps.length) * 100}%` }} />
          </div>
        </div>
      </div>
      <Card className="space-y-3 p-5">
        <h2 className="font-display text-lg font-semibold">{s.title}</h2>
        <div className="space-y-2 text-[15px]">
          <Markdown remarkPlugins={[remarkGfm]} components={md}>{s.body}</Markdown>
        </div>
        {s.command && <Code text={s.command} />}
      </Card>
      <div className="flex justify-between">
        <Button variant="ghost" onClick={() => setStep((n) => Math.max(0, n - 1))} disabled={step === 0}>Back</Button>
        {step < last ? (
          <Button variant="primary" icon={<ChevronRight className="size-4" />} onClick={() => setStep((n) => n + 1)}>Next</Button>
        ) : (
          <Button variant="primary" icon={<Check className="size-4" />} onClick={() => { onDone(); onBack(); }}>Finish</Button>
        )}
      </div>
    </div>
  );
}

function LessonList({ kids, done, onOpen }: { kids: boolean; done: string[]; onOpen: (l: Lesson) => void }) {
  const list = lessons.filter((l) => !!l.kids === kids);
  return (
    <div className="grid gap-3 sm:grid-cols-2">
      {list.map((l) => (
        <button key={l.id} onClick={() => onOpen(l)} className="rounded-xl border border-line bg-surface p-4 text-left shadow-[var(--shadow)] transition hover:bg-surface-2">
          <div className="flex items-center justify-between">
            <div className="flex size-9 items-center justify-center rounded-lg bg-accent-soft text-accent">
              {l.kids ? <Rocket className="size-5" /> : <GraduationCap className="size-5" />}
            </div>
            {done.includes(l.id) && <Badge tone="good"><BadgeCheck className="size-3.5" /> Done</Badge>}
          </div>
          <h3 className="mt-3 font-display font-semibold">{l.title}</h3>
          <p className="mt-1 text-sm text-muted">{l.summary}</p>
          <p className="mt-2 text-xs text-muted">{l.steps.length} steps · about {l.minutes} min</p>
        </button>
      ))}
    </div>
  );
}

// ---- Quests ----

function ChallengeRow({ c, index, solved, onSolved }: { c: Quest["challenges"][number]; index: number; solved: boolean; onSolved: () => void }) {
  const [value, setValue] = useState("");
  const [wrong, setWrong] = useState(false);
  const [showHint, setShowHint] = useState(false);

  const check = async (v: string) => {
    setValue(v);
    setWrong(false);
    if (!v.trim()) return;
    if ((await hashFlag(v)) === c.hash) onSolved();
    else if (v.trim().length >= 4 && v.includes("}")) setWrong(true);
  };

  return (
    <Card className={`p-4 ${solved ? "border-good/40 bg-good-soft/30" : ""}`}>
      <div className="flex items-start gap-3">
        <div className={`mt-0.5 flex size-6 shrink-0 items-center justify-center rounded-full text-xs font-semibold ${solved ? "bg-good text-white" : "bg-surface-2 text-muted"}`}>
          {solved ? <Check className="size-4" /> : index + 1}
        </div>
        <div className="min-w-0 flex-1 space-y-2">
          <div className="text-sm font-medium">{c.title}</div>
          <div className="space-y-2 text-sm text-muted [&_code]:text-ink">
            <Markdown remarkPlugins={[remarkGfm]} components={md}>{c.prompt}</Markdown>
          </div>
          {solved ? (
            <p className="text-sm font-medium text-good">Solved!</p>
          ) : (
            <>
              <div className="flex gap-2">
                <input
                  value={value}
                  onChange={(e) => check(e.target.value)}
                  placeholder="Type the flag you found"
                  spellCheck={false}
                  className={`h-9 w-full max-w-xs rounded-lg border bg-surface px-3 font-mono text-sm outline-none transition focus:ring-2 focus:ring-accent/25 ${wrong ? "border-bad" : "border-line focus:border-accent"}`}
                />
                <button onClick={() => setShowHint((h) => !h)} className="flex items-center gap-1 rounded-lg px-2 text-xs text-muted hover:text-ink">
                  <Lightbulb className="size-4" /> Hint
                </button>
              </div>
              {wrong && <p className="text-xs text-bad">Not quite — keep looking.</p>}
              {showHint && <p className="rounded-lg bg-surface-2 px-3 py-2 text-xs text-ink">{c.hint}</p>}
            </>
          )}
        </div>
      </div>
    </Card>
  );
}

function QuestView({ quest, distros, solved, onSolve, onBack, onGetLinux }: {
  quest: Quest; distros: Distro[]; solved: number[]; onSolve: (i: number) => void; onBack: () => void; onGetLinux: () => void;
}) {
  const fits = (d: Distro) => quest.systems.includes("any") || (d.info.catalogId != null && quest.systems.includes(d.info.catalogId));
  const usable = distros.filter(fits);
  const [target, setTarget] = useState(usable[0]?.name ?? "");
  const [setting, setSetting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [ready, setReady] = useState(false);

  const setup = async () => {
    setSetting(true);
    setError(null);
    try {
      await api.startQuest(target, quest.id);
      setReady(true);
    } catch (e) {
      setError(String(e));
    }
    setSetting(false);
  };

  const allDone = solved.length === quest.challenges.length;
  const systemNames = quest.systems.includes("any") ? "any system" : quest.systems.map((s) => (s === "kali" ? "Kali" : s === "parrot" ? "Parrot" : s)).join(" or ");

  return (
    <div className="max-w-2xl space-y-5">
      <button onClick={onBack} className="flex items-center gap-1.5 text-sm text-muted hover:text-ink">
        <ArrowLeft className="size-4" /> All challenges
      </button>
      <div>
        <h1 className="font-display text-2xl font-semibold">{quest.title}</h1>
        <p className="mt-1 text-muted">{quest.summary}</p>
      </div>

      <Notice>
        <div className="space-y-1 text-sm [&_code]:text-ink">
          <Markdown remarkPlugins={[remarkGfm]} components={md}>{quest.intro}</Markdown>
        </div>
      </Notice>

      <Card className="space-y-3 p-5">
        <h2 className="flex items-center gap-2 font-display font-semibold"><Terminal className="size-4 text-accent" /> Set it up</h2>
        {usable.length === 0 ? (
          <Notice tone="warn">
            This quest needs {systemNames}. You do not have one yet.
            <div className="mt-2"><Button onClick={onGetLinux}>Get Linux</Button></div>
          </Notice>
        ) : (
          <>
            <p className="text-sm text-muted">
              EasyWSL builds a practice folder (<code className="rounded bg-surface-2 px-1 font-mono text-[13px]">{quest.folder}</code>) in your
              system and opens a terminal there. Then solve each challenge below and type in the flags you find.
            </p>
            <div className="flex flex-wrap items-center gap-2">
              <span className="text-sm">Run it in</span>
              <select value={target} onChange={(e) => setTarget(e.target.value)} className="h-9 rounded-lg border border-line bg-surface px-3 text-sm outline-none focus:border-accent">
                {usable.map((d) => <option key={d.name} value={d.name}>{d.name}</option>)}
              </select>
              <Button variant="primary" icon={<Play className="size-4" />} busy={setting} onClick={setup}>
                {ready ? "Set up again" : "Set up and open terminal"}
              </Button>
            </div>
            {ready && <p className="text-sm text-good">Ready in {target}. The terminal is open at {quest.folder}.</p>}
            {error && <Notice tone="bad">{error}</Notice>}
          </>
        )}
      </Card>

      <div className="flex items-center justify-between">
        <h2 className="font-display font-semibold">Challenges</h2>
        <span className="text-sm text-muted">{solved.length} / {quest.challenges.length} solved</span>
      </div>

      {allDone && (
        <Notice tone="good" icon={<BadgeCheck className="size-5 text-good" />}>
          <b>Quest complete!</b> You solved every challenge. {quest.kids ? "Amazing work, explorer!" : "Nicely done."}
        </Notice>
      )}

      <div className="space-y-2.5">
        {quest.challenges.map((c, i) => (
          <ChallengeRow key={i} c={c} index={i} solved={solved.includes(i)} onSolved={() => onSolve(i)} />
        ))}
      </div>
    </div>
  );
}

function QuestList({ kids, solvedByQuest, onOpen }: { kids: boolean; solvedByQuest: Record<string, number[]>; onOpen: (q: Quest) => void }) {
  const list = quests.filter((q) => !!q.kids === kids);
  return (
    <div className="grid gap-3 sm:grid-cols-2">
      {list.map((q) => {
        const n = solvedByQuest[q.id]?.length ?? 0;
        const total = q.challenges.length;
        return (
          <button key={q.id} onClick={() => onOpen(q)} className="rounded-xl border border-line bg-surface p-4 text-left shadow-[var(--shadow)] transition hover:bg-surface-2">
            <div className="flex items-center justify-between">
              <div className="flex size-9 items-center justify-center rounded-lg bg-accent-soft text-accent">
                {q.kids ? <Rocket className="size-5" /> : q.id === "kali" ? <Lock className="size-5" /> : <Swords className="size-5" />}
              </div>
              {n === total ? <Badge tone="good"><BadgeCheck className="size-3.5" /> Complete</Badge> : n > 0 ? <Badge tone="accent">{n}/{total}</Badge> : null}
            </div>
            <h3 className="mt-3 font-display font-semibold">{q.title}</h3>
            <p className="mt-1 text-sm text-muted">{q.summary}</p>
            <p className="mt-2 text-xs text-muted">{total} challenges · flags look like <code className="font-mono">{q.flagShape}</code></p>
          </button>
        );
      })}
    </div>
  );
}

// ---- Page ----

export default function Learn({ distros, onGetLinux }: { distros: Distro[]; onGetLinux: () => void }) {
  const [tab, setTab] = useState<"lessons" | "quests">("lessons");
  const [kids, setKids] = useKidsMode();
  const [lessonsDone, setLessonsDone] = useState<string[]>(() => load(lessonsKey, []));
  const [solved, setSolved] = useState<Record<string, number[]>>(() => load(questKey("all"), {}));
  const [openLesson, setOpenLesson] = useState<Lesson | null>(null);
  const [openQuest, setOpenQuest] = useState<Quest | null>(null);

  useEffect(() => save(lessonsKey, lessonsDone), [lessonsDone]);
  useEffect(() => save(questKey("all"), solved), [solved]);

  // Leaving the list open while switching kids mode could show an item that no longer fits.
  useEffect(() => { setOpenLesson(null); setOpenQuest(null); }, [kids]);

  const markLessonDone = (id: string) => setLessonsDone((d) => (d.includes(id) ? d : [...d, id]));
  const solveChallenge = (questId: string, i: number) =>
    setSolved((s) => {
      const have = s[questId] ?? [];
      return have.includes(i) ? s : { ...s, [questId]: [...have, i] };
    });

  if (openLesson) return <LessonView lesson={openLesson} onBack={() => setOpenLesson(null)} onDone={() => markLessonDone(openLesson.id)} />;
  if (openQuest)
    return (
      <QuestView
        quest={openQuest}
        distros={distros}
        solved={solved[openQuest.id] ?? []}
        onSolve={(i) => solveChallenge(openQuest.id, i)}
        onBack={() => setOpenQuest(null)}
        onGetLinux={onGetLinux}
      />
    );

  return (
    <div className="mx-auto max-w-3xl space-y-6">
      <header className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h1 className="font-display text-2xl font-semibold">{kids ? "Learn — Kids" : "Learn"}</h1>
          <p className="text-muted">{kids ? "Friendly projects and adventures to learn the computer." : "Guided lessons and hands-on challenges for Linux, the terminal and security."}</p>
        </div>
        <button
          onClick={() => setKids(!kids)}
          className={`inline-flex items-center gap-2 rounded-lg border px-3 h-9 text-sm font-medium transition ${kids ? "border-accent bg-accent-soft text-accent" : "border-line hover:bg-surface-2"}`}
        >
          <Baby className="size-4" /> Kids mode{kids ? " on" : ""}
        </button>
      </header>

      <div className="flex gap-1 rounded-lg bg-surface-2 p-1 w-fit">
        {(["lessons", "quests"] as const).map((t) => (
          <button
            key={t}
            onClick={() => setTab(t)}
            className={`rounded-md px-4 py-1.5 text-sm font-medium transition ${tab === t ? "bg-surface text-ink shadow-sm" : "text-muted hover:text-ink"}`}
          >
            {t === "lessons" ? "Lessons" : "Challenges"}
          </button>
        ))}
      </div>

      {tab === "lessons" ? (
        <LessonList kids={kids} done={lessonsDone} onOpen={setOpenLesson} />
      ) : (
        <QuestList kids={kids} solvedByQuest={solved} onOpen={setOpenQuest} />
      )}

      {tab === "quests" && !kids && (
        <Notice icon={<CircleHelp className="size-4 text-accent" />}>
          Challenges build harmless practice files in a system you choose, then you hunt for flags in the terminal. The Security
          Basics quest is defensive only — reading evidence, never attacking anything. <b>Only ever test systems you own or have
          written permission to test.</b>
        </Notice>
      )}
    </div>
  );
}
