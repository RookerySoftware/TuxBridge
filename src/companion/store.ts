// What the companion remembers between runs: whether the first-launch tour was seen, the new
// systems it should show around once they are ready, the pages it has introduced, and the stars a
// child earned in its lessons. Kept in localStorage, like the other preferences (see prefs.ts).

const KEYS = {
  tourDone: "easywsl.companion.tourDone",
  showAround: "easywsl.companion.showAround",
  seenPages: "easywsl.companion.seenPages",
  stars: "easywsl.companion.stars",
  pointing: "easywsl.companion.pointing",
};

function read<T>(key: string, fallback: T): T {
  try {
    const raw = localStorage.getItem(key);
    return raw ? (JSON.parse(raw) as T) : fallback;
  } catch {
    return fallback;
  }
}

function write(key: string, value: unknown) {
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch {
    /* private mode, ignore */
  }
}

export interface ShowAround {
  /** The system's name, as in My systems. */
  name: string;
  /** Its catalogue id ("ubuntu"), which picks what the tour says. */
  catalogId: string;
}

export const tourDone = () => read(KEYS.tourDone, false);
export const setTourDone = () => write(KEYS.tourDone, true);
const tourListeners = new Set<() => void>();
/** Starts the first-launch tour again (Settings). */
export function startAppTour() {
  write(KEYS.tourDone, false);
  for (const l of tourListeners) l();
}
export function onStartAppTour(l: () => void) {
  tourListeners.add(l);
  return () => {
    tourListeners.delete(l);
  };
}

export const pendingShowArounds = () => read<ShowAround[]>(KEYS.showAround, []);
/** Asked for by the install form's "Show me around": the tour starts once the system is ready. */
export function requestShowAround(name: string, catalogId: string) {
  write(KEYS.showAround, [...pendingShowArounds().filter((s) => s.name !== name), { name, catalogId }]);
}
export function finishShowAround(name: string) {
  write(KEYS.showAround, pendingShowArounds().filter((s) => s.name !== name));
}

export const seenPages = () => read<string[]>(KEYS.seenPages, []);
export const markPageSeen = (page: string) => write(KEYS.seenPages, [...new Set([...seenPages(), page])]);

export const stars = () => read<string[]>(KEYS.stars, []);
export const addStar = (lesson: string) => write(KEYS.stars, [...new Set([...stars(), lesson])]);

/** Whether pointing at a button makes the companion explain it. */
export const pointing = () => read(KEYS.pointing, true);
export const setPointing = (on: boolean) => write(KEYS.pointing, on);
