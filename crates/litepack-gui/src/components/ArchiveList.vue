<script setup lang="ts">
import { ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import FileDropZone from "./FileDropZone.vue";
import { listArchive } from "../api/tauri";
import type { EntryInfo } from "../types";

const archives = ref<string[]>([]);
const entries = ref<EntryInfo[]>([]);
const loading = ref(false);
const error = ref("");

const archiveFilters = [
  { name: "压缩包", extensions: ["zip", "7z"] },
];

function formatSize(n: number | null): string {
  if (n == null) return "—";
  const units = ["B", "KB", "MB", "GB"];
  let v = n;
  let i = 0;
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024;
    i++;
  }
  return `${v.toFixed(v >= 10 || i === 0 ? 0 : 1)} ${units[i]}`;
}

async function inspect() {
  error.value = "";
  if (!archives.value.length) return;
  loading.value = true;
  try {
    entries.value = await listArchive(archives.value[0]);
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

async function pickArchive() {
  const picked = await open({
    multiple: false,
    directory: false,
    filters: archiveFilters,
    title: "选择压缩包",
  });
  if (picked && !Array.isArray(picked)) {
    archives.value = [picked];
    await inspect();
  }
}
</script>

<template>
  <div class="panel">
    <div class="field">
      <label>压缩包</label>
      <div class="control">
        <input
          type="text"
          v-model="archives[0]"
          placeholder="选择或拖入压缩包"
        />
        <button type="button" @click="pickArchive">浏览…</button>
        <button type="button" :disabled="!archives.length || loading" @click="inspect">
          {{ loading ? "读取中…" : "读取内容" }}
        </button>
      </div>
    </div>
    <FileDropZone
      v-model="archives"
      :filters="archiveFilters"
      pick-title="选择要查看的压缩包"
      hint="拖拽一个 ZIP / 7z 压缩包"
    />
    <div v-if="entries.length" class="file-list" style="max-height: 340px">
      <table class="table">
        <thead>
          <tr>
            <th>名称</th>
            <th>大小</th>
            <th>压缩后</th>
            <th>方法</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="e in entries" :key="e.path">
            <td class="name" :title="e.path">
              {{ e.is_dir ? "📁 " : "" }}{{ e.path }}
            </td>
            <td>{{ formatSize(e.size) }}</td>
            <td>{{ formatSize(e.compressed_size) }}</td>
            <td>{{ e.method ?? "—" }}</td>
          </tr>
        </tbody>
      </table>
    </div>
    <div v-if="error" class="err">{{ error }}</div>
  </div>
</template>
