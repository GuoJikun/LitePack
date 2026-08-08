import { defineStore } from "pinia";
import type { EntryInfo, ProgressReport } from "../types";

export type TaskKind = "compress" | "extract";

interface TaskRecord {
  id: number;
  kind: TaskKind;
  report: ProgressReport | null;
  error?: string;
}

export const useAppStore = defineStore("app", {
  state: () => ({
    theme: (localStorage.getItem("litepack-theme") as "light" | "dark") ?? "light",
    archive: null as string | null,
    archiveName: "" as string,
    archiveFormat: "" as string,
    entries: [] as EntryInfo[],
    currentDir: "" as string,
    search: "",
    selected: [] as string[],
    sortKey: "name" as "name" | "size" | "modified",
    sortAsc: true,
    task: null as TaskRecord | null,
  }),
  getters: {
    /** 当前目录下的直接子条目（目录去重）。 */
    children(): EntryInfo[] {
      const prefix = this.currentDir;
      const map = new Map<string, EntryInfo>();
      for (const e of this.entries) {
        if (!e.path.startsWith(prefix)) continue;
        const rest = e.path.slice(prefix.length);
        if (!rest) continue;
        const firstSlash = rest.indexOf("/");
        if (firstSlash === -1) {
          // 直接子文件
          map.set(rest, e);
        } else {
          // 子目录名（目录条目或文件条目的父级）
          const dirName = rest.slice(0, firstSlash);
          const dirPath = prefix + dirName + "/";
          if (!map.has(dirName)) {
            map.set(dirName, {
              path: dirPath,
              size: 0,
              is_dir: true,
              compressed_size: null,
              method: null,
              modified: null,
              encrypted: false,
            });
          }
        }
      }
      let list = [...map.values()];
      if (this.search) {
        const q = this.search.toLowerCase();
        list = list.filter(
          (e) =>
            e.path.toLowerCase().includes(q) ||
            (!e.is_dir && (e.method ?? "").toLowerCase().includes(q)),
        );
      }
      list.sort((a, b) => {
        if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1;
        let r = 0;
        if (this.sortKey === "size") {
          r = a.size - b.size;
        } else if (this.sortKey === "modified") {
          r = (a.modified ?? 0) - (b.modified ?? 0);
        } else {
          r = a.path.localeCompare(b.path);
        }
        return this.sortAsc ? r : -r;
      });
      return list;
    },
    breadcrumbs(): { name: string; path: string }[] {
      const parts = this.currentDir.split("/").filter(Boolean);
      const out = [{ name: "根目录", path: "" }];
      let acc = "";
      for (const p of parts) {
        acc += p + "/";
        out.push({ name: p, path: acc });
      }
      return out;
    },
    dirTotalSize(): number {
      const prefix = this.currentDir;
      return this.entries
        .filter((e) => e.path.startsWith(prefix) && !e.is_dir)
        .reduce((sum, e) => sum + e.size, 0);
    },
    dirTotalCompressed(): number {
      const prefix = this.currentDir;
      return this.entries
        .filter((e) => e.path.startsWith(prefix) && !e.is_dir)
        .reduce((sum, e) => sum + (e.compressed_size ?? 0), 0);
    },
  },
  actions: {
    setTheme(t: "light" | "dark") {
      this.theme = t;
      localStorage.setItem("litepack-theme", t);
      document.documentElement.dataset.theme = t;
    },
    toggleTheme() {
      this.setTheme(this.theme === "light" ? "dark" : "light");
    },
    openArchive(path: string, name: string, format: string) {
      this.archive = path;
      this.archiveName = name;
      this.archiveFormat = format;
      this.currentDir = "";
      this.entries = [];
      this.search = "";
      this.selected = [];
    },
    closeArchive() {
      this.archive = null;
      this.archiveName = "";
      this.archiveFormat = "";
      this.entries = [];
      this.currentDir = "";
      this.search = "";
      this.selected = [];
    },
    navigate(dirPath: string) {
      this.currentDir = dirPath;
      this.selected = [];
    },
    toggleSelect(path: string) {
      const i = this.selected.indexOf(path);
      if (i >= 0) this.selected.splice(i, 1);
      else this.selected.push(path);
    },
    startTask(id: number, kind: TaskKind) {
      this.task = { id, kind, report: null };
    },
    updateProgress(report: ProgressReport) {
      if (this.task) this.task.report = report;
    },
    finishTask(id: number, ok: boolean, message?: string) {
      if (this.task?.id === id) {
        if (ok) this.task = null;
        else {
          this.task.error = message;
        }
      }
    },
    clearTaskError() {
      if (this.task) this.task.error = undefined;
    },
  },
});
