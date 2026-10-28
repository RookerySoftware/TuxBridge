// Searching the guides. Every guide is split at its ## and ### headings, and a query is matched
// against those parts, so a result can open the guide right at the part that answers it.
//
// Queries understand operators, like a web search:
//   words           every word must appear (in the part, or the guide's title and summary)
//   "two words"     those words together, in that order
//   -word  -"a b"   leave out parts that have it
//   a OR b, a | b   either one
//   inst*           any word starting with "inst"
//   title:word      in a guide's title or a heading
//   in:ubuntu       only in that guide (its file name or title); guide: works too
//   section:docker  only in that section of the list ("Using Linux", "Docker"...)
//   code:apt        inside a command or code example
// Words are matched loosely ("installing" finds "install"), common synonyms count a little
// ("delete" finds "remove"), and a word found nowhere is corrected to the nearest one that exists.

import { guides, type Guide } from "./guides";

export interface Part {
  guide: Guide;
  /** The heading, or "" for the start of the guide. */
  heading: string;
  /** The id of the heading in the rendered guide. */
  anchor: string;
  /** The part as written, in Markdown. */
  markdown: string;
  /** The text without Markdown. */
  text: string;
  /** The text of its commands and code. */
  code: string;
  stems: Map<string, number>;
  titleStems: Set<string>;
  codeStems: Set<string>;
  length: number;
}

/** "Installing software" -> "installing-software", the same as Help gives its headings. */
export function slug(text: string): string {
  return text.toLowerCase().replace(/[`*_]/g, "").replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "");
}

/** Markdown -> readable text. */
function plain(md: string): string {
  return md
    .replace(/```[^\n]*\n/g, "")
    .replace(/```/g, "")
    .replace(/!\[[^\]]*\]\([^)]*\)/g, "")
    .replace(/\[([^\]]+)\]\([^)]*\)/g, "$1")
    .replace(/^#+\s*/gm, "")
    .replace(/^\s*\|?\s*-{3,}.*$/gm, "")
    .replace(/^>\s?/gm, "")
    .replace(/[*`|]/g, " ")
    .replace(/[ \t]+/g, " ")
    .replace(/\n{2,}/g, "\n")
    .trim();
}

function codeOf(md: string): string {
  const blocks = [...md.matchAll(/```[^\n]*\n([\s\S]*?)```/g)].map((m) => m[1]);
  const inline = [...md.replace(/```[\s\S]*?```/g, "").matchAll(/`([^`\n]+)`/g)].map((m) => m[1]);
  return [...blocks, ...inline].join("\n");
}

/** Lower-case words, keeping things like "wsl.conf", "c++" and "apt-get" whole. */
export function tokens(text: string): string[] {
  return text.toLowerCase().match(/[a-z0-9][a-z0-9.+_-]*[a-z0-9+]|[a-z0-9]/g) ?? [];
}

/** A light stemmer: "installing", "installed", "installs" -> "install"; "update", "updating" -> "updat". */
export function stem(word: string): string {
  if (word.length <= 3 || /[^a-z]/.test(word)) return word;
  let w = word;
  for (const end of ["ations", "ation", "ings", "ing", "ies", "ied", "ed", "es", "s"]) {
    if (!w.endsWith(end) || w.length - end.length < 3) continue;
    const base = w.slice(0, -end.length);
    // "boxes" -> "box", but "updates" -> "update" (below); "process" keeps its s.
    if (end === "es" && !/(s|x|z|ch|sh)$/.test(base)) continue;
    if (end === "s" && base.endsWith("s")) continue;
    if (end === "ies" || end === "ied") w = base + "y";
    // "running" -> "run", "stopped" -> "stop"
    else if ((end === "ing" || end === "ed") && base.length > 3 && base[base.length - 1] === base[base.length - 2] && !base.endsWith("ss") && !base.endsWith("ll")) w = base.slice(0, -1);
    else w = base;
    break;
  }
  // "update" and "updat(ing)" alike.
  if (w.length >= 4 && w.endsWith("e")) w = w.slice(0, -1);
  return w;
}

/** Words people use for the same thing. Matches through a synonym count half. */
const SYNONYMS: string[][] = [
  ["remove", "delete", "uninstall", "erase"],
  ["folder", "directory", "dir"],
  ["program", "app", "application", "software", "package"],
  ["update", "upgrade"],
  ["restart", "reboot"],
  ["stop", "kill", "end", "quit", "shutdown"],
  ["start", "launch", "run", "open"],
  ["error", "problem", "fail", "broken", "issue"],
  ["password", "passphrase"],
  ["user", "account"],
  ["internet", "network", "networking", "online"],
  ["disk", "drive", "storage", "space"],
  ["copy", "duplicate"],
  ["move", "rename"],
  ["terminal", "shell", "console", "command-line"],
  ["desktop", "gui"],
  ["memory", "ram"],
  ["speed", "fast", "slow", "performance"],
  ["backup", "export"],
  ["admin", "root", "sudo", "superuser"],
  ["windows", "win"],
  ["schedule", "daily", "hourly", "weekly", "cron", "timer"],
  ["customize", "customise", "personalize", "theme", "prettier", "prompt"],
  ["slow", "lag", "sluggish"],
  ["editor", "edit"],
];
const synonymsOf = new Map<string, string[]>();
for (const group of SYNONYMS) {
  const stems = group.map(stem);
  for (const s of stems) synonymsOf.set(s, [...(synonymsOf.get(s) ?? []), ...stems.filter((x) => x !== s)]);
}

/** Words that say nothing about the subject. */
const STOP = new Set(
  ("a an the and or to of in on at by is it be do i me my we you your are can how what with that this from have " +
    "not but all use does did into out its was will when then them they there their which where who why get got " +
    "just like want need make some any one more about also only should would could may other than very way help " +
    "please thanks thank know so if as").split(" "),
);

function count(list: string[]): Map<string, number> {
  const m = new Map<string, number>();
  for (const w of list) m.set(w, (m.get(w) ?? 0) + 1);
  return m;
}

export const parts: Part[] = guides.flatMap((g) => {
  const pieces = g.body.split(/\n(?=#{2,3} )/);
  return pieces.map((piece) => {
    const head = piece.match(/^#{2,3} (.*)/);
    const heading = head ? head[1].trim() : "";
    const body = head ? piece.slice(head[0].length) : piece.replace(/^# .*\n+/, "");
    const text = plain(body);
    const code = codeOf(body);
    const titleWords = tokens(`${g.title} ${heading}`).map(stem);
    const all = [...tokens(text).map(stem), ...titleWords];
    return {
      guide: g,
      heading,
      anchor: heading ? slug(heading) : "",
      markdown: body.trim(),
      text,
      code,
      stems: count(all),
      titleStems: new Set(titleWords),
      codeStems: new Set(tokens(code).map(stem)),
      length: all.length,
    };
  });
});

const averageLength = parts.reduce((n, p) => n + p.length, 0) / Math.max(parts.length, 1);
const documentFrequency = count(parts.flatMap((p) => [...p.stems.keys()]));
const vocabulary = [...documentFrequency.keys()];
/** Each guide's title and summary, which every part of it shares. */
const guideStems = new Map(guides.map((g) => [g.id, new Set(tokens(`${g.title} ${g.summary}`).map(stem))]));

// ---- The query ----

type Field = "any" | "title" | "code";

interface Term {
  /** One word (stemmed), or a phrase (lower-case, as typed). */
  value: string;
  phrase: boolean;
  prefix: boolean;
  field: Field;
}

export interface Query {
  /** Each clause must match; a clause matches when one of its terms does (OR). */
  clauses: Term[][];
  exclude: Term[];
  guides: string[];
  sections: string[];
  /** Words that were corrected: typed -> used. */
  corrected: [string, string][];
  /** Words to highlight in results and the guide. */
  highlight: string[];
  /** Set by searchGuides when no part had every word, so the closest are shown. */
  loose?: boolean;
}

function term(raw: string, field: Field): Term | null {
  if (raw.startsWith('"')) {
    const value = raw.replace(/^"|"$/g, "").toLowerCase().trim();
    if (!value) return null;
    return tokens(value).length > 1 ? { value, phrase: true, prefix: false, field } : { value: stem(tokens(value)[0] ?? value), phrase: false, prefix: false, field };
  }
  const prefix = raw.endsWith("*");
  const word = tokens(raw.replace(/\*$/, ""))[0];
  if (!word) return null;
  return { value: prefix ? word : stem(word), phrase: false, prefix, field };
}

function editDistance(a: string, b: string, max: number): number {
  if (Math.abs(a.length - b.length) > max) return max + 1;
  let prev = Array.from({ length: b.length + 1 }, (_, i) => i);
  for (let i = 1; i <= a.length; i++) {
    const row = [i];
    let best = i;
    for (let j = 1; j <= b.length; j++) {
      row[j] = Math.min(prev[j] + 1, row[j - 1] + 1, prev[j - 1] + (a[i - 1] === b[j - 1] ? 0 : 1));
      best = Math.min(best, row[j]);
    }
    if (best > max) return max + 1;
    prev = row;
  }
  return prev[b.length];
}

/** The word in the guides nearest to one found nowhere, if there is a close one. */
function nearest(word: string): string | null {
  if (word.length < 4 || /[^a-z]/.test(word)) return null;
  const max = word.length >= 7 ? 2 : 1;
  let best: string | null = null;
  let bestScore = Infinity;
  for (const v of vocabulary) {
    const d = editDistance(word, v, max);
    if (d > max) continue;
    // Closer first, then the more common word.
    const score = d * 1000 - (documentFrequency.get(v) ?? 0);
    if (score < bestScore) {
      bestScore = score;
      best = v;
    }
  }
  return best;
}

export function parseQuery(input: string): Query {
  const q: Query = { clauses: [], exclude: [], guides: [], sections: [], corrected: [], highlight: [] };
  let pieces: string[] = [...(input.match(/-?[a-z]+:"[^"]*"?|-?"[^"]*"?|\S+/gi) ?? [])];
  // A question in plain words ("how do I install python") needs only the words that matter.
  const meaningful = pieces.filter((p) => !STOP.has(p.toLowerCase()));
  if (meaningful.some((p) => p !== "OR" && p !== "|")) pieces = meaningful;
  let joinNext = false;
  for (const piece of pieces) {
    if (piece === "OR" || piece === "|") {
      joinNext = q.clauses.length > 0;
      continue;
    }
    // -word leaves it out; --option (as in a command) is just a word.
    const negative = /^-[^-\s]/.test(piece);
    let raw = negative ? piece.slice(1) : piece;
    let field: Field = "any";
    const op = raw.match(/^([a-z]+):(.*)$/i);
    if (op && op[2]) {
      const name = op[1].toLowerCase();
      const value = op[2].replace(/^"|"$/g, "").toLowerCase();
      if (name === "in" || name === "guide") {
        q.guides.push(value);
        continue;
      }
      if (name === "section") {
        q.sections.push(value);
        continue;
      }
      if (name === "title" || name === "heading") field = "title";
      else if (name === "code" || name === "cmd" || name === "command") field = "code";
      if (field !== "any") raw = op[2];
    }
    const t = term(raw, field);
    if (!t) continue;
    if (negative) {
      q.exclude.push(t);
      continue;
    }
    // A word found nowhere: use the nearest one that exists.
    if (!t.phrase && !t.prefix && !documentFrequency.has(t.value)) {
      const near = nearest(t.value);
      if (near) {
        q.corrected.push([raw.replace(/^"|"$/g, ""), near]);
        t.value = near;
      }
    }
    q.highlight.push(t.phrase ? t.value : raw.replace(/[*"]/g, "").toLowerCase());
    if (joinNext) q.clauses[q.clauses.length - 1].push(t);
    else q.clauses.push([t]);
    joinNext = false;
  }
  return q;
}

/** How well `t` matches part `p`: 0 for not at all. */
function match(t: Term, p: Part): number {
  const has = (s: string) => {
    if (t.field === "title") return p.titleStems.has(s) ? (p.stems.get(s) ?? 1) : 0;
    if (t.field === "code") return p.codeStems.has(s) ? (p.stems.get(s) ?? 1) : 0;
    return p.stems.get(s) ?? 0;
  };
  if (t.phrase) {
    const hay = t.field === "code" ? p.code.toLowerCase() : t.field === "title" ? `${p.guide.title} ${p.heading}`.toLowerCase() : `${p.heading}\n${p.text}`.toLowerCase();
    const n = hay.split(t.value).length - 1;
    return n ? 2.5 * n : 0;
  }
  const words = t.prefix ? vocabulary.filter((v) => v.startsWith(t.value)) : [t.value];
  let score = 0;
  for (const w of words) score += bm25(w, has(w), p);
  if (!t.prefix && t.field === "any") {
    for (const s of synonymsOf.get(t.value) ?? []) score += 0.5 * bm25(s, has(s), p);
  }
  if (score > 0 && t.field !== "title" && words.some((w) => p.titleStems.has(w))) score *= 1.8;
  return score;
}

function bm25(word: string, tf: number, p: Part): number {
  if (!tf) return 0;
  const df = documentFrequency.get(word) ?? 0;
  const idf = Math.log(1 + (parts.length - df + 0.5) / (df + 0.5));
  return (idf * tf * 2.2) / (tf + 1.2 * (0.25 + 0.75 * (p.length / averageLength)));
}

/** Whether `t` is found in the guide's title or summary, which count for each of its parts. */
function inGuide(t: Term, g: Guide): boolean {
  if (t.field !== "any") return false;
  if (t.phrase) return `${g.title} ${g.summary}`.toLowerCase().includes(t.value);
  const s = guideStems.get(g.id)!;
  return t.prefix ? [...s].some((w) => w.startsWith(t.value)) : s.has(t.value);
}

export interface Hit {
  part: Part;
  score: number;
}

export interface GuideResult {
  guide: Guide;
  score: number;
  /** Its best parts, best first. */
  hits: Hit[];
}

export function hasFilters(q: Query) {
  return q.guides.length > 0 || q.sections.length > 0;
}

/**
 * The parts matching `q`, grouped by guide, best first. When no part has every word of a plain
 * question, it settles for the parts with the most of them, and sets `q.loose`.
 */
export function searchGuides(q: Query): GuideResult[] {
  const strict = collect(q, false);
  const plainWords = q.clauses.length > 1 && q.clauses.every((c) => c.length === 1 && c[0].field === "any");
  q.loose = strict.length === 0 && plainWords;
  return q.loose ? collect(q, true) : strict;
}

function collect(q: Query, loose: boolean): GuideResult[] {
  const byGuide = new Map<string, GuideResult>();
  // in:arch means the Arch guide, not every guide whose name contains "arch".
  const isGuide = (g: Guide, x: string) => (guides.some((o) => o.id === x) ? g.id === x : g.id.startsWith(x) || g.title.toLowerCase().includes(x));
  for (const p of parts) {
    const g = p.guide;
    if (q.guides.length && !q.guides.some((x) => isGuide(g, x))) continue;
    if (q.sections.length && !q.sections.some((x) => g.section.toLowerCase().includes(x))) continue;
    if (q.exclude.some((t) => match(t, p) > 0)) continue;
    let score = 0;
    let found = 0;
    let ok = true;
    for (const clause of q.clauses) {
      const best = Math.max(...clause.map((t) => match(t, p)));
      if (best > 0) {
        score += best;
        found++;
      } else if (clause.some((t) => inGuide(t, g))) score += 0.3;
      else if (!loose) {
        ok = false;
        break;
      }
    }
    if (!ok || (loose && found === 0)) continue;
    // Loosely, a part with more of the words beats one with a single word many times.
    if (loose) score *= found / q.clauses.length;
    // Only filters (in:ubuntu): every part, in order.
    if (q.clauses.length === 0) score = 1;
    if (score <= 0) continue;
    const r = byGuide.get(g.id) ?? { guide: g, score: 0, hits: [] };
    r.hits.push({ part: p, score });
    byGuide.set(g.id, r);
  }
  for (const r of byGuide.values()) {
    r.hits.sort((a, b) => b.score - a.score);
    // A guide with several good parts beats one with a single lucky one, a little.
    r.score = r.hits[0].score + 0.25 * r.hits.slice(1, 4).reduce((n, h) => n + h.score, 0);
  }
  return [...byGuide.values()].sort((a, b) => b.score - a.score);
}

/** Plain BM25 over the parts, for the assistant: the parts that best answer a question. */
export function bestParts(question: string, limit: number): Part[] {
  const terms = [...new Set(tokens(question).filter((w) => w.length > 1 && !STOP.has(w)).map(stem))];
  return parts
    .map((p) => ({ p, score: terms.reduce((n, t) => n + bm25(t, p.stems.get(t) ?? 0, p) * (p.titleStems.has(t) ? 1.8 : 1), 0) }))
    .filter((x) => x.score > 0)
    .sort((a, b) => b.score - a.score)
    .slice(0, limit)
    .map((x) => x.p);
}

/** About `width` characters of `text` around the first highlighted word. */
export function snippet(text: string, words: string[], width = 170): string {
  const lower = text.toLowerCase();
  let at = -1;
  for (const w of words) {
    const i = lower.indexOf(w.toLowerCase());
    if (i >= 0 && (at < 0 || i < at)) at = i;
  }
  if (at < 0) return text.slice(0, width).replace(/\n/g, " ") + (text.length > width ? "…" : "");
  let start = Math.max(0, at - 50);
  // Start at a whole word.
  if (start > 0) {
    const space = text.indexOf(" ", start);
    if (space >= 0 && space < at) start = space + 1;
  }
  const end = Math.min(text.length, start + width);
  return (start > 0 ? "…" : "") + text.slice(start, end).replace(/\n/g, " ") + (end < text.length ? "…" : "");
}

/** The words to highlight: what was typed, and the forms of each that the guides use. */
export function highlightWords(q: Query): string[] {
  const out = new Set<string>();
  for (const h of q.highlight) {
    out.add(h);
    const s = stem(h);
    if (s.length >= 3) out.add(s);
  }
  for (const [typed, used] of q.corrected) {
    out.delete(typed.toLowerCase());
    out.add(used);
  }
  return [...out].filter((w) => w.length >= 2);
}
