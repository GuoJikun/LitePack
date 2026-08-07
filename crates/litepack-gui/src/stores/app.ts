import { defineStore } from "pinia";
import type { ProgressReport } from "../types";

export type TaskKind = "compress" | "extract";
export type TabKind = "compress" | "extract" | "list";

interface TaskRecord {
  id: number;
  kind: TaskKind;
  status: "running" | "done" | "error";
  message?: string;
}

export const useAppStore = defineStore("app", {
  state: () => ({
    theme: (localStorage.getItem("litepack-theme") as "light" | "dark") ?? "light",
    tab: "compress" as TabKind,
    active: null as null | {
      id: number;
      kind: TaskKind;
      report: ProgressReport | null;
    },
    history: [] as TaskRecord[],
  }),
  actions: {
    setTheme(t: "light" | "dark") {
      this.theme = t;
      localStorage.setItem("litepack-theme", t);
      document.documentElement.dataset.theme = t;
    },
    toggleTheme() {
      this.setTheme(this.theme === "light" ? "dark" : "light");
    },
    startTask(id: number, kind: TaskKind) {
      this.active = { id, kind, report: null };
      this.history.unshift({ id, kind, status: "running" });
      if (this.history.length > 50) this.history.pop();
    },
    updateProgress(report: ProgressReport) {
      if (this.active) this.active.report = report;
    },
    finishTask(id: number, ok: boolean, message?: string) {
      const kind = this.active?.id === id ? this.active.kind : "compress";
      if (this.active?.id === id) this.active = null;
      const rec = this.history.find((r) => r.id === id);
      if (rec) {
        rec.status = ok ? "done" : "error";
        rec.message = message;
      }
    },
  },
});
