// Live data for the pages: distros, jobs and desktop sessions, refreshed by the backend's events.

import { useCallback, useEffect, useRef, useState } from "react";
import { api, on, type Distro, type Job, type Session } from "./api";

export function useDistros() {
  const [distros, setDistros] = useState<Distro[] | null>(null);
  const [responding, setResponding] = useState(true);
  // One question to WSL at a time: when it hangs, refreshes must not pile up behind it.
  const inFlight = useRef(false);
  const refresh = useCallback(() => {
    if (inFlight.current) return;
    inFlight.current = true;
    api
      .listDistros()
      .then((l) => {
        setDistros(l.distros);
        setResponding(l.responding);
      })
      .catch(() => setDistros((d) => d ?? []))
      .finally(() => (inFlight.current = false));
  }, []);
  useEffect(() => {
    refresh();
    const off = on("distros-changed", refresh);
    // Running/stopped changes from outside the app (a terminal closing, WSL stopping an idle
    // system) have no event; a slow poll catches them.
    const timer = setInterval(refresh, 5000);
    return () => {
      off();
      clearInterval(timer);
    };
  }, [refresh]);
  return { distros, responding, refresh };
}

export function useJobs() {
  const [jobs, setJobs] = useState<Job[]>([]);
  useEffect(() => {
    api.listJobs().then(setJobs);
    return on<Job>("job", (job) =>
      setJobs((list) => {
        const i = list.findIndex((j) => j.id === job.id);
        if (i === -1) return [...list, job];
        const copy = list.slice();
        copy[i] = job;
        return copy;
      }),
    );
  }, []);
  const dismiss = useCallback((id: number) => {
    api.dismissJob(id);
    setJobs((list) => list.filter((j) => j.id !== id));
  }, []);
  return { jobs, dismiss };
}

export function useSessions() {
  const [sessions, setSessions] = useState<Record<string, Session>>({});
  useEffect(() => {
    api.listSessions().then((list) => setSessions(Object.fromEntries(list.map((s) => [s.distro, s]))));
    return on<Session>("session", (s) =>
      setSessions((map) => {
        const copy = { ...map };
        if (s.state === "ended") delete copy[s.distro];
        else copy[s.distro] = s;
        return copy;
      }),
    );
  }, []);
  const clear = useCallback((distro: string) => setSessions((map) => {
    const copy = { ...map };
    delete copy[distro];
    return copy;
  }), []);
  return { sessions, clear };
}
