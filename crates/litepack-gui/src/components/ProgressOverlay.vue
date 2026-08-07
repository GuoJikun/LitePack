<script setup lang="ts">
import { computed } from "vue";
import { useAppStore } from "../stores/app";
import { cancelOperation } from "../api/tauri";

const store = useAppStore();

const phaseText = computed(() => {
  switch (store.active?.report?.phase) {
    case "Listing":
      return "正在读取目录…";
    case "Compressing":
      return "正在压缩…";
    case "Extracting":
      return "正在解压…";
    default:
      return "准备中…";
  }
});

const percent = computed(() => {
  const r = store.active?.report;
  if (!r || !r.total_bytes) return 0;
  return Math.min(100, Math.round((r.done_bytes / r.total_bytes) * 100));
});

function formatSize(n: number): string {
  const units = ["B", "KB", "MB", "GB"];
  let v = n;
  let i = 0;
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024;
    i++;
  }
  return `${v.toFixed(v >= 10 || i === 0 ? 0 : 1)} ${units[i]}`;
}
</script>

<template>
  <div v-if="store.active" class="overlay">
    <div class="card">
      <div class="row">
        <div class="grow">
          <strong>{{ phaseText }}</strong>
        </div>
        <span class="hint">
          {{ store.active.report ? formatSize(store.active.report.done_bytes) : "0 B" }}
          / {{ store.active.report ? formatSize(store.active.report.total_bytes) : "0 B" }}
        </span>
      </div>
      <div class="progress-bar">
        <div class="fill" :style="{ width: percent + '%' }" />
      </div>
      <div class="row">
        <span class="hint grow" style="overflow: hidden; text-overflow: ellipsis; white-space: nowrap">
          {{ store.active.report?.current_file ?? "" }}
        </span>
        <button type="button" class="danger" @click="cancelOperation(store.active!.id)">
          取消
        </button>
      </div>
    </div>
  </div>
</template>
