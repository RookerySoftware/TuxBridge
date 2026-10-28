// Opening the Feedback page from elsewhere (a failed task's "Report this"), with a message filled
// in. App.tsx listens and switches to the page; the page takes the draft once.

let draft: string | null = null;
const listeners = new Set<() => void>();

/** Opens the Feedback page as a bug report, with `text` as the start of the message. */
export function reportProblem(text: string) {
  draft = text;
  for (const l of listeners) l();
}

export function onReportProblem(l: () => void) {
  listeners.add(l);
  return () => {
    listeners.delete(l);
  };
}

/** The waiting draft, if any; taking it clears it. */
export function takeDraft(): string | null {
  const d = draft;
  draft = null;
  return d;
}
