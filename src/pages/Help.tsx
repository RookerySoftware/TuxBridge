// The help directory: guides written in Markdown (help/*.md at the top of the repository, shared
// with the website), with a list by section, a search, and links between guides.

import { useEffect, useMemo, useRef, useState } from "react";
import Markdown, { type Components } from "react-markdown";
import remarkGfm from "remark-gfm";
import { openUrl } from "@tauri-apps/plugin-opener";
import { Search } from "lucide-react";
import { inApp } from "../api";
import { inputClass } from "../components/ui";
import { SECTIONS, guides, type Guide } from "../guides";

const openExternal = (url: string) => (inApp ? openUrl(url) : window.open(url));

function article(onLink: (id: string) => void): Components {
  return {
    h1: ({ children }) => <h1 className="font-display text-2xl font-semibold">{children}</h1>,
    h2: ({ children }) => <h2 className="pt-3 font-display text-lg font-semibold">{children}</h2>,
    h3: ({ children }) => <h3 className="pt-2 font-semibold">{children}</h3>,
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
    pre: ({ children }) => <pre className="overflow-x-auto rounded-lg bg-surface-2 px-4 py-3">{children}</pre>,
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
          else onLink(href.replace(/^\.\//, "").replace(/\.md(#.*)?$/, ""));
        }}
      >
        {children}
      </a>
    ),
  };
}

export default function Help({ topic, onTopic }: { topic: string | null; onTopic: (id: string) => void }) {
  const [query, setQuery] = useState("");
  const current = guides.find((g) => g.id === topic) ?? guides[0];
  const scroller = useRef<HTMLDivElement>(null);
  useEffect(() => {
    scroller.current?.scrollTo(0, 0);
  }, [current?.id]);

  const found = useMemo(() => {
    const words = query.toLowerCase().split(/\s+/).filter(Boolean);
    if (words.length === 0) return null;
    return guides.filter((g) => {
      const text = `${g.title} ${g.summary} ${g.body}`.toLowerCase();
      return words.every((w) => text.includes(w));
    });
  }, [query]);

  if (!current) return <p className="text-muted">No guides yet.</p>;
  const sections = SECTIONS.filter((s) => guides.some((g) => g.section === s));

  return (
    <div className="-mx-8 -my-7 flex h-[calc(100%+3.5rem)]">
      <nav className="w-64 shrink-0 space-y-5 overflow-y-auto border-r border-line px-4 py-6">
        <div className="relative">
          <Search className="pointer-events-none absolute left-2.5 top-2.5 size-4 text-muted" />
          <input className={`${inputClass} pl-8`} placeholder="Search the guides" value={query} onChange={(e) => setQuery(e.target.value)} />
        </div>
        {found ? (
          <div className="space-y-0.5">
            {found.length === 0 && <p className="px-2 text-sm text-muted">Nothing found.</p>}
            {found.map((g) => (
              <GuideLink key={g.id} guide={g} active={g.id === current.id} onClick={() => onTopic(g.id)} />
            ))}
          </div>
        ) : (
          sections.map((section) => (
            <div key={section} className="space-y-0.5">
              <h3 className="px-2 pb-1 text-xs font-semibold uppercase tracking-wide text-muted">{section}</h3>
              {guides
                .filter((g) => g.section === section)
                .map((g) => (
                  <GuideLink key={g.id} guide={g} active={g.id === current.id} onClick={() => onTopic(g.id)} />
                ))}
            </div>
          ))
        )}
      </nav>
      <div ref={scroller} className="flex-1 overflow-y-auto px-10 py-8">
        <article className="max-w-2xl space-y-3.5 text-[15px]">
          <Markdown remarkPlugins={[remarkGfm]} components={article(onTopic)}>
            {current.body}
          </Markdown>
        </article>
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
