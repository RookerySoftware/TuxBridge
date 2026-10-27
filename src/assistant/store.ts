// The conversation with the assistant. It lives outside the pages so it survives moving between
// them, and is kept in this browser's storage so it survives closing the app.

import { useSyncExternalStore } from "react";
import { api, on, type AiChunk, type AiMessage } from "../api";
import { instructions } from "./context";

export interface ChatMessage {
  role: "user" | "assistant";
  content: string;
  /** Set when the answer did not finish. */
  error?: string;
}

interface State {
  messages: ChatMessage[];
  /** The answer being written, if any. */
  answering: number | null;
  /** Text waiting in the input box (from "Ask the assistant" elsewhere in the app). */
  draft: string;
  /** Whether the assistant is installed, for the buttons elsewhere that ask it. */
  installed: boolean;
}

const KEY = "easywsl.assistant.chat";
/** The last messages sent with a question; older ones fall out of what the model sees. */
const HISTORY = 12;

function load(): ChatMessage[] {
  try {
    const saved = JSON.parse(localStorage.getItem(KEY) ?? "[]");
    return Array.isArray(saved) ? saved.filter((m) => m && (m.role === "user" || m.role === "assistant")) : [];
  } catch {
    return [];
  }
}

let state: State = { messages: load(), answering: null, draft: "", installed: false };
const listeners = new Set<() => void>();

function set(change: Partial<State>) {
  state = { ...state, ...change };
  if (change.messages) {
    try {
      localStorage.setItem(KEY, JSON.stringify(state.messages.slice(-100)));
    } catch {
      // Storage full or blocked: the chat still works, it is just not kept.
    }
  }
  listeners.forEach((l) => l());
}

export function useAssistant(): State {
  return useSyncExternalStore(
    (l) => {
      listeners.add(l);
      return () => listeners.delete(l);
    },
    () => state,
  );
}

/** Appends to the answer being written. */
function append(change: (m: ChatMessage) => ChatMessage) {
  const messages = state.messages.slice();
  messages[messages.length - 1] = change(messages[messages.length - 1]);
  set({ messages });
}

on<AiChunk>("ai-chunk", (chunk) => {
  if (chunk.id !== state.answering) return;
  if (chunk.text) append((m) => ({ ...m, content: m.content + chunk.text }));
  if (chunk.done) {
    if (chunk.error) append((m) => ({ ...m, error: chunk.error! }));
    set({ answering: null });
  }
});

let nextId = Date.now();

export async function send(text: string) {
  const question = text.trim();
  if (!question || state.answering !== null) return;
  const id = ++nextId;
  const before = state.messages.filter((m) => !m.error || m.content);
  set({ messages: [...before, { role: "user", content: question }, { role: "assistant", content: "" }], answering: id, draft: "" });
  try {
    const history: AiMessage[] = [...before, { role: "user" as const, content: question }]
      .slice(-HISTORY)
      .map((m) => ({ role: m.role, content: m.content }));
    const system = await instructions(history);
    await api.aiChat(id, [{ role: "system", content: system }, ...history]);
  } catch (e) {
    append((m) => ({ ...m, error: String(e) }));
    set({ answering: null });
  }
}

export function stop() {
  if (state.answering === null) return;
  api.aiCancel(state.answering);
  set({ answering: null });
}

export function newChat() {
  stop();
  set({ messages: [] });
}

export function setDraft(draft: string) {
  set({ draft });
}

export function setInstalled(installed: boolean) {
  if (installed !== state.installed) set({ installed });
}

let navigate: (() => void) | null = null;

/** The app's way to show the assistant's page. */
export function onOpen(show: () => void) {
  navigate = show;
}

/** Opens the assistant with this question ready to send. */
export function askAbout(question: string) {
  set({ draft: question });
  navigate?.();
}

/** Shows the assistant's page. */
export function openAssistant() {
  navigate?.();
}
