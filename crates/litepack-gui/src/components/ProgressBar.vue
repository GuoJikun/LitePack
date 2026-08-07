<script setup lang="ts">
import { computed } from "vue";
import { useAppStore } from "../stores/app";
import { cancelOperation } from "../api/tauri";
import { humanSize } from "../utils/format";

const store = useAppStore();

const phaseText = computed(() => {
  switch (store.task?.report?.phase) {
    case "Listing":
      return "正在读取…";
    case "Compressing":
      return "正在压缩…";
    case "Extracting":
      return "正在解压…";
    default:
      return "准备中…";
  }
});

const percent = computed(() => {
  const r = store.task?.report;
  if (!r || !r.total_bytes) return 0;
  return Math.min(100, Math.round((r.done_bytes / r.total_bytes) * 100));
});

const bytesText = computed(() => {
  const r = store.task?.report;
  if (!r) return "";
  return `${humanSize(r.done_bytes)} / ${humanSize(r.total_bytes)}`;
});
</script>

<template>
  <div v-if="store.task" class="progress-bar">
    <div class="progress-inner">
      <div class="progress-top">
        <span class="progress-phase">
          <span class="progress-spinner" />
          {{ phaseText }}
        </span>
        <span class="progress-bytes">{{ bytesText }}</span>
      </div>
      <div class="progress-track">
        <div class="progress-fill" :style="{ width: percent + '%' }" />
      </div>
      <div class="progress-bottom">
        <span class="progress-file">{{ store.task.report?.current_file ?? "" }}</span>
        <button class="progress-cancel" @click="cancelOperation(store.task!.id)">取消</button>
      </div>
    </div>
  </div>
</template>
