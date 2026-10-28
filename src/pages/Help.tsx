// The help directory: guides written in Markdown (help/*.md at the top of the repository, shared
// with the website), with a list by section, a search with operators (search.ts), and links
// between guides. A topic is a guide's id, optionally with a heading: "ubuntu#common-problems".

import { Children, isValidElement, useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import Markdown, { type Components } from "react-markdown";
import remarkGfm from "remark-gfm";
import { openUrl } from "@tauri-apps/plugin-opener";
import { ArrowLeft, CircleHelp, Search, X } from "lucide-react";
import { inApp } from "../api";
import { inputClass } from "../components/ui";
import { SECTIONS, guides, type Guide } from "../guides";
import { hasFilters, highlightWords, parseQuery, searchGuides, slug, snippet, type Hit } from "../search";

const openExternal = (url: string) => (inApp ? openUrl(url) : window.open(url));

/** The text inside rendered Markdown children. */
function textOf(node: ReactNode): string {
  return Children.toArray(node)
    .map((c) => (typeof c === "string" || typeof c === "number" ? String(c) : isValidElement<{ children?: ReactNode }>(c) ? textOf(c.props.children) : ""))
    .join("");
}

function article(onLink: (id: string) => void): Components {
  return {
    h1: ({ children }) => <h1 className="font-display text-2xl font-semibold">{children}</h1>,
    h2: ({ children }) => (
      <h2 id={slug(textOf(children))} className="scroll-mt-6 pt-3 font-display text-lg font-semibold">
        {children}
      </h2>
    ),
    h3: ({ children }) => (
      <h3 id={slug(textOf(children))} className="scroll-mt-6 pt-2 font-semibold">
        {children}
      </h3>
    ),
    p: ({ children }) => <p className="leading-relaxed">{children}</p>,
    ul: ({ children }) => <ul className="list-disc space-y-1 pl-5 leading-relaxed">{children}</ul>,
    ol: ({ children }) => <ol className="list-decimal space-y-1 pl-5 leading-relaxed">{children}</ol>,
    blockquote: ({ children }) => <div className="rounded-lg bg-accent-soft px-4 py-3 [&>p]:text-ink">{children}</div>,
    code: ({ className, children }) =>
      className ? (
        <code className="font-mono text-[13px]">{children}</code>
      ) : (
        <code className="rounded bg-surface-2 px-1 py-0.5 font-mono text-[13px]">{children}</code>
      ),
    pre: ({ children }) => <pre className="select-text overflow-x-auto rounded-lg bg-surface-2 px-4 py-3">{children}</pre>,
    table: ({ children }) => (
      <div className="overflow-x-auto">
        <table className="w-full border-collapse text-sm">{children}</table>
      </div>
    ),
    th: ({ children }) => <th className="border-b border-line px-2 py-1.5 text-left font-medium">{children}</th>,
    td: ({ children }) => <td className="border-b border-line px-2 py-1.5 align-top">{children}</td>,
    a: ({ href, children }) => (
      <a
        href={href}
        className="text-accent underline-offset-2 hover:underline"
        onClick={(e) => {
          e.preventDefault();
          if (!href) return;
          if (/^https?:/.test(href)) openExternal(href);
          else if (href.startsWith("#")) document.getElementById(href.slice(1))?.scrollIntoView({ behavior: "smooth" });
          else onLink(href.replace(/^\.\//, "").replace(/\.md(?=#|$)/, ""));
        }}
      >
        {children}
      </a>
    ),
  };
}

/** The search operators, for the "?" panel. Clicking an example puts it in the search. */
const OPERATORS: [string, string][] = [
  ["install docker", "Parts with every word (in any order)"],
  ['"open files"', "Those words together, in that order"],
  ["update -arch", "Leave out parts that mention a word"],
  ["apt OR pacman", "Either word (| works too)"],
  ["perm*", "Words starting with perm: permission, permanent…"],
  ["title:password", "In a guide's title or a heading"],
  ["in:ubuntu snap", "Only in one guide (its name, or part of its title)"],
  ['section:"using linux" copy', "Only in one section of the list"],
  ["code:apt-get", "Inside a command or code example"],
];

/** Marks the search words in `text`. */
function Marked({ text, words }: { text: string; words: string[] }) {
  if (!words.length) return <>{text}</>;
  const re = new RegExp(`(${words.map((w) => w.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")).join("|")})`, "gi");
  return (
    <>
      {text.split(re).map((piece, i) =>
        i % 2 ? (
          <mark key={i} className="rounded-sm bg-warn-soft px-0.5 text-ink">
            {piece}
          </mark>
        ) : (
          piece
        ),
      )}
    </>
  );
}

/** Highlights the search words in the open guide (CSS Custom Highlight API, where available). */
function useArticleHighlight(root: React.RefObject<HTMLElement | null>, words: string[], key: string) {
  useEffect(() => {
    const highlights = (CSS as unknown as { highlights?: Map<string, unknown> }).highlights;
    const HighlightCtor = (window as unknown as { Highlight?: new (...r: Range[]) => unknown }).Highlight;
    if (!highlights || !HighlightCtor) return;
    highlights.delete("guide-search");
    if (!root.current || !words.length) return;
    const ranges: Range[] = [];
    const walker = document.createTreeWalker(root.current, NodeFilter.SHOW_TEXT);
    const lower = words.map((w) => w.toLowerCase());
    for (let node = walker.nextNode(); node; node = walker.nextNode()) {
      const text = node.textContent?.toLowerCase() ?? "";
      for (const w of lower) {
        for (let i = text.indexOf(w); i >= 0; i = text.indexOf(w, i + w.length)) {
          const r = new Range();
          r.setStart(node, i);
          r.setEnd(node, i + w.length);
          ranges.push(r);
        }
      }
    }
    highlights.set("guide-search", new HighlightCtor(...ranges));
    return () => {
      highlights.delete("guide-search");
    };
  }, [root, words, key]);
}

export default function Help({ topic, onTopic }: { topic: string | null; onTopic: (id: string) => void }) {
  const [query, setQuery] = useState("");
  const [showResults, setShowResults] = useState(false);
  const [selected, setSelected] = useState(0);
  const [operators, setOperators] = useState(false);
  const [id, anchor] = (topic ?? "").split("#");
  const current = guides.find((g) => g.id === id) ?? guides[0];
  const scroller = useRef<HTMLDivElement>(null);
  const articleRef = useRef<HTMLElement>(null);
  const input = useRef<HTMLInputElement>(null);
  // Made once, so the article isn't rebuilt (and its highlights lost) on every render.
  const components = useMemo(() => article(onTopic), [onTopic]);

  const parsed = useMemo(() => parseQuery(query), [query]);
  const active = parsed.clauses.length > 0 || hasFilters(parsed);
  const results = useMemo(() => (active ? searchGuides(parsed) : []), [parsed, active]);
  const words = useMemo(() => (active ? highlightWords(parsed) : []), [parsed, active]);
  const flat = useMemo(() => results.flatMap((r) => r.hits.slice(0, 3)), [results]);
  const resultCount = results.reduce((n, r) => n + r.hits.length, 0);

  useEffect(() => setSelected(0), [query]);

  // The guide opens at the top, or at the heading asked for.
  useEffect(() => {
    if (anchor) document.getElementById(anchor)?.scrollIntoView({ block: "start" });
    else scroller.current?.scrollTo(0, 0);
  }, [current?.id, anchor, showResults]);

  useArticleHighlight(articleRef, showResults ? [] : words, `${current?.id}#${anchor}`);

  // Ctrl+K or / jumps to the search.
  useEffect(() => {
    const key = (e: KeyboardEvent) => {
      const typing = e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement;
      if ((e.ctrlKey && e.key.toLowerCase() === "k") || (e.key === "/" && !typing)) {
        e.preventDefault();
        input.current?.focus();
        input.current?.select();
      }
    };
    window.addEventListener("keydown", key);
    return () => window.removeEventListener("keydown", key);
  }, []);

  const open = (hit: Hit) => {
    onTopic(hit.part.anchor ? `${hit.part.guide.id}#${hit.part.anchor}` : hit.part.guide.id);
    setShowResults(false);
  };

  if (!current) return <p className="text-muted">No guides yet.</p>;
  const sections = SECTIONS.filter((s) => guides.some((g) => g.section === s));
  const matching = new Set(results.map((r) => r.guide.id));

  return (
    <div className="-mx-8 -my-7 flex h-[calc(100%+3.5rem)]">
      <nav className="w-64 shrink-0 space-y-5 overflow-y-auto border-r border-line px-4 py-6">
        <div className="space-y-1.5">
          <div className="relative">
            <Search className="pointer-events-none absolute left-2.5 top-2.5 size-4 text-muted" />
            <input
              ref={input}
              className={`${inputClass} pl-8 pr-14`}
              placeholder="Search the guides"
              value={query}
              spellCheck={false}
              onChange={(e) => {
                setQuery(e.target.value);
                setShowResults(true);
              }}
              onFocus={() => active && setShowResults(true)}
              onKeyDown={(e) => {
                if (e.key === "ArrowDown") {
                  e.preventDefault();
                  setShowResults(true);
                  setSelected((s) => Math.min(s + 1, flat.length - 1));
                } else if (e.key === "ArrowUp") {
                  e.preventDefault();
                  setSelected((s) => Math.max(s - 1, 0));
                } else if (e.key === "Enter" && flat[selected]) {
                  open(flat[selected]);
                } else if (e.key === "Escape") {
                  setQuery("");
                  setShowResults(false);
                }
              }}
            />
            <div className="absolute right-1.5 top-1.5 flex">
              {query && (
                <button aria-label="Clear the search" className="rounded p-1 text-muted hover:text-ink" onClick={() => { setQuery(""); setShowResults(false); }}>
                  <X className="size-4" />
                </button>
              )}
              <button
                aria-label="Search operators"
                title="Search operators"
                className={`rounded p-1 hover:text-ink ${operators ? "text-accent" : "text-muted"}`}
                onClick={() => setOperators(!operators)}
              >
                <CircleHelp className="size-4" />
              </button>
            </div>
          </div>
          <p className="px-1 text-[11px] text-muted">Ctrl+K or / to search · ↑↓ and Enter to open</p>
        </div>

        {operators && (
          <div className="space-y-1.5 rounded-lg border border-line bg-surface p-3 text-xs animate-in">
            <p className="font-medium">Search operators</p>
            {OPERATORS.map(([example, meaning]) => (
              <button
                key={example}
                className="block w-full rounded px-1 py-0.5 text-left hover:bg-surface-2"
                onClick={() => {
                  setQuery(example);
                  setShowResults(true);
                  input.current?.focus();
                }}
              >
                <code className="font-mono text-accent">{example}</code>
                <span className="block text-muted">{meaning}</span>
              </button>
            ))}
            <p className="pt-1 text-muted">Combine them freely. Spelling slips are corrected, and words like delete and remove count as the same.</p>
          </div>
        )}

        {sections.map((section) => {
          const list = guides.filter((g) => g.section === section && (!active || matching.has(g.id)));
          if (!list.length) return null;
          return (
            <div key={section} className="space-y-0.5">
              <h3 className="px-2 pb-1 text-xs font-semibold uppercase tracking-wide text-muted">{section}</h3>
              {list.map((g) => (
                <GuideLink
                  key={g.id}
                  guide={g}
                  active={!showResults && g.id === current.id}
                  onClick={() => {
                    onTopic(g.id);
                    setShowResults(false);
                  }}
                />
              ))}
            </div>
          );
        })}
      </nav>

      <div ref={scroller} className="flex-1 overflow-y-auto px-10 py-8">
        {active && showResults ? (
          <div className="max-w-2xl space-y-5">
            <div>
              <h1 className="font-display text-2xl font-semibold">Search</h1>
              <p className="text-muted">
                {resultCount === 0 ? "Nothing found" : `${resultCount} ${resultCount === 1 ? "place" : "places"} in ${results.length} ${results.length === 1 ? "guide" : "guides"}`}
                {parsed.loose && resultCount > 0 && " · no place has every word, so these have the most of them"}
                {parsed.corrected.length > 0 && (
                  <>
                    {" "}· searched for{" "}
                    {parsed.corrected.map(([typed, used], i) => (
                      <span key={typed}>
                        {i > 0 && ", "}
                        <b className="text-ink">{used}</b> instead of <s>{typed}</s>
                      </span>
                    ))}
                  </>
                )}
              </p>
            </div>
            {resultCount === 0 && (
              <div className="space-y-2 text-sm text-muted">
                <p>Try fewer words, a shorter form of a word with <code className="font-mono">*</code> (like <code className="font-mono">instal*</code>), or <code className="font-mono">OR</code> between alternatives.</p>
                <p>Or <b className="text-ink">Ask Pip</b>, who can also answer from what Pip knows.</p>
              </div>
            )}
            {results.map((r) => (
              <section key={r.guide.id} className="space-y-1.5">
                <button className="text-left font-display font-semibold hover:text-accent" onClick={() => open(r.hits.find((h) => !h.part.heading) ?? r.hits[0])}>
                  {r.guide.title}
                  <span className="ml-2 text-xs font-normal text-muted">{r.guide.section}</span>
                </button>
                {r.hits.slice(0, 3).map((h) => {
                  const index = flat.indexOf(h);
                  return (
                    <button
                      key={h.part.anchor || "top"}
                      onClick={() => open(h)}
                      onMouseEnter={() => setSelected(index)}
                      className={`block w-full rounded-lg border px-3 py-2 text-left transition ${index === selected ? "border-accent bg-accent-soft" : "border-line hover:bg-surface-2"}`}
                    >
                      <span className="block text-sm font-medium">
                        <Marked text={h.part.heading || r.guide.summary || r.guide.title} words={words} />
                      </span>
                      <span className="block text-sm leading-relaxed text-muted">
                        <Marked text={snippet(h.part.text, words)} words={words} />
                      </span>
                    </button>
                  );
                })}
                {r.hits.length > 3 && <p className="px-1 text-xs text-muted">and {r.hits.length - 3} more in this guide</p>}
              </section>
            ))}
          </div>
        ) : (
          <>
            {active && (
              <button className="mb-4 inline-flex items-center gap-1 text-sm text-accent hover:underline" onClick={() => setShowResults(true)}>
                <ArrowLeft className="size-4" /> Back to {resultCount} results for “{query.trim()}”
              </button>
            )}
            <article ref={articleRef} className="max-w-2xl space-y-3.5 text-[15px]">
              <Markdown remarkPlugins={[remarkGfm]} components={components}>
                {current.body}
              </Markdown>
            </article>
          </>
        )}
      </div>
    </div>
  );
}

function GuideLink({ guide, active, onClick }: { guide: Guide; active: boolean; onClick: () => void }) {
  return (
    <button
      onClick={onClick}
      title={guide.summary}
      className={`block w-full rounded-lg px-2 py-1.5 text-left text-sm transition ${active ? "bg-accent-soft font-medium text-accent" : "hover:bg-surface-2"}`}
    >
      {guide.title}
    </button>
  );
}
