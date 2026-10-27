// What the assistant is told with every question: how EasyWSL works (knowledge.md), what is on
// this PC right now, and the sections of the guides that match the question. A small model
// answers well about what it is shown, so finding the right sections matters more than its size.

import { api, type AiMessage, type Distro } from "../api";
import { guides } from "../guides";
import knowledge from "./knowledge.md?raw";

interface Section {
  guide: string;
  title: string;
  text: string;
  words: Map<string, number>;
  length: number;
}

/** Words that say nothing about the subject. */
const COMMON = new Set(
  ("the and for you your are can how what with that this from have not but all use does into out its was will " +
    "when then them they there their which where who why get got just like want need make some any one more " +
    "about also only should would could may other than very way help please thanks thank know").split(" "),
);

function words(text: string): string[] {
  return (text.toLowerCase().match(/[a-z0-9][a-z0-9.+_-]*[a-z0-9]|[a-z0-9]/g) ?? []).filter((w) => w.length > 1 && !COMMON.has(w));
}

function count(list: string[]): Map<string, number> {
  const m = new Map<string, number>();
  for (const w of list) m.set(w, (m.get(w) ?? 0) + 1);
  return m;
}

/** Each guide split at its ## headings, the guide's title and summary kept with every part. */
const sections: Section[] = guides.flatMap((g) => {
  const parts = g.body.split(/\n(?=## )/);
  return parts.map((part) => {
    const heading = part.startsWith("## ") ? part.slice(3, part.indexOf("\n")).trim() : "";
    const title = heading ? `${g.title}: ${heading}` : g.title;
    // The title counts three times: a section about "Docker" is about Docker.
    const list = [...words(part), ...words(`${title} ${g.summary}`), ...words(title), ...words(title)];
    return { guide: g.id, title, text: part.replace(/^# .*\n+/, "").trim(), words: count(list), length: list.length };
  });
});
const averageLength = sections.reduce((n, s) => n + s.length, 0) / Math.max(sections.length, 1);
const documentFrequency = count(sections.flatMap((s) => [...s.words.keys()]));

/** BM25: the sections that best match the question, best first. */
export function search(question: string, limit: number): Section[] {
  const terms = [...new Set(words(question))];
  const scored = sections.map((s) => {
    let score = 0;
    for (const t of terms) {
      const tf = s.words.get(t) ?? 0;
      if (!tf) continue;
      const df = documentFrequency.get(t) ?? 0;
      const idf = Math.log(1 + (sections.length - df + 0.5) / (df + 0.5));
      score += (idf * tf * 2.2) / (tf + 1.2 * (0.25 + 0.75 * (s.length / averageLength)));
    }
    return { s, score };
  });
  return scored
    .filter((x) => x.score > 0)
    .sort((a, b) => b.score - a.score)
    .slice(0, limit)
    .map((x) => x.s);
}

function describeSystem(d: Distro): string {
  const parts = [d.catalog?.title ?? "a Linux system EasyWSL did not install"];
  if (d.info.user) parts.push(`user ${d.info.user}`);
  const desktops = [d.info.desktop && `${d.catalog?.desktop?.name ?? d.info.desktop} desktop`, d.info.hyprland && "Hyprland desktop"].filter(Boolean);
  parts.push(desktops.length ? desktops.join(" and ") : "no desktop");
  parts.push(d.running ? "running" : "stopped");
  if (d.isDefault) parts.push("the default");
  if (d.wslVersion === 1) parts.push("WSL 1 (old; most things need WSL 2)");
  if (d.info.docker) parts.push("its docker uses EasyWSL's engine");
  return `- ${d.name}: ${parts.join(", ")}`;
}

/** What is on this PC now; whatever does not answer quickly is left out. */
async function thisPc(): Promise<string> {
  const quick = <T,>(p: Promise<T>) => Promise.race([p, new Promise<null>((r) => setTimeout(() => r(null), 3000))]).catch(() => null);
  const [listing, docker] = await Promise.all([quick(api.listDistros()), quick(api.dockerStatus())]);
  const lines: string[] = [];
  if (listing) {
    if (!listing.responding) lines.push("WSL is not responding right now (My systems offers Restart WSL).");
    lines.push(listing.distros.length ? "Linux systems:" : "No Linux systems installed yet.");
    lines.push(...listing.distros.map(describeSystem));
  }
  if (docker) {
    if (!docker.installed) lines.push(`Docker: not turned on in EasyWSL${docker.desktopInstalled ? " (Docker Desktop is installed)" : ""}.`);
    else lines.push(`Docker: EasyWSL's engine ${docker.version ?? ""} is ${docker.running ? "running" : "stopped"}.`);
    if (docker.installed && docker.running) {
      const containers = await quick(api.containers());
      if (containers?.length) {
        lines.push("Containers:");
        for (const c of containers.slice(0, 15)) {
          lines.push(`- ${c.name} (${c.image}): ${c.state}${c.ports.length ? `, at localhost:${c.ports.join(", localhost:")}` : ""}`);
        }
      }
    }
  }
  return lines.join("\n");
}

/** Up to this many characters of guides (about 2,000 words), so the chat itself has room. */
const GUIDE_BUDGET = 9000;

/** The instructions for an answer to the last question in `history`. */
export async function instructions(history: AiMessage[]): Promise<string> {
  // A follow-up ("and on Arch?") is about the question before it too.
  const asked = history.filter((m) => m.role === "user").slice(-2).map((m) => m.content).join("\n");
  const found = search(asked, 6);
  let budget = GUIDE_BUDGET;
  const parts: string[] = [];
  for (const s of found) {
    const text = s.text.length > budget ? s.text.slice(0, budget) + " …" : s.text;
    if (budget < 400) break;
    budget -= text.length;
    parts.push(`From the guide "${s.title}" (${s.guide}.md):\n${text}`);
  }
  return [
    "You are the EasyWSL assistant. You help with EasyWSL, Linux, WSL, Windows and Docker, and politely decline other subjects.",
    knowledge.trim(),
    `# This PC right now\n\n${await thisPc()}`,
    parts.length ? `# Guides that match the question\n\n${parts.join("\n\n---\n\n")}` : "# Guides\n\nNo guide matches this question closely; answer from what you know, and say when you are unsure.",
  ].join("\n\n");
}
