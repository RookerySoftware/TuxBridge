// The guides: Markdown files in help/ at the top of the repository (shared with the website),
// read into the app when it is built.

export interface Guide {
  /** The file name without .md: "files", "ubuntu". */
  id: string;
  title: string;
  summary: string;
  section: string;
  order: number;
  body: string;
}

/** The sections, in the order the list shows them. */
export const SECTIONS = [
  "Start here",
  "Using Linux",
  "Programming",
  "WSL and Windows",
  "Desktops",
  "Docker",
  "Your system",
  "When something goes wrong",
  "Reference",
];

const files = import.meta.glob("../help/*.md", { query: "?raw", import: "default", eager: true }) as Record<string, string>;

/** "---\nkey: value\n---\nbody" -> fields and body. */
function parse(path: string, text: string): Guide {
  const id = path.split("/").pop()!.replace(/\.md$/, "");
  const fields: Record<string, string> = {};
  let body = text.replace(/\r\n/g, "\n");
  const match = body.match(/^---\n([\s\S]*?)\n---\n/);
  if (match) {
    for (const line of match[1].split("\n")) {
      const i = line.indexOf(":");
      if (i > 0) fields[line.slice(0, i).trim()] = line.slice(i + 1).trim();
    }
    body = body.slice(match[0].length);
  }
  return {
    id,
    title: fields.title ?? id,
    summary: fields.summary ?? "",
    section: fields.section ?? "Using Linux",
    order: Number(fields.order ?? 50),
    body,
  };
}

export const guides: Guide[] = Object.entries(files)
  .map(([path, text]) => parse(path, text))
  .sort((a, b) => a.order - b.order || a.title.localeCompare(b.title));
