<script setup lang="ts">
import { computed } from "vue";
import type { ProgressReport } from "../types";

const props = defineProps<{
  progress: ProgressReport | null;
  error?: string;
}>();

const emit = defineEmits<{
  (e: "cancel"): void;
}>();

const percent = computed(() => {
  if (!props.progress || props.progress.total_bytes === 0) return 0;
  return Math.min(100, Math.round((props.progress.done_bytes / props.progress.total_bytes) * 100));
});

const currentFileName = computed(() => {
  if (!props.progress) return "";
  const parts = props.progress.current_file.replace(/\\/g, "/").split("/");
  return parts[parts.length - 1] || props.progress.current_file;
});

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return (bytes / Math.pow(1024, i)).toFixed(1) + " " + units[i];
}
</script>

<template>
  <div class="mini-progress">
    <div class="mini-progress-titlebar">
      <span class="mini-progress-title">
        {{ error ? "解压失败" : "解压中..." }}
      </span>
      <div class="mini-progress-buttons">
        <button class="titlebar-btn" @click="emit('cancel')">
          <svg viewBox="0 0 12 12" width="12" height="12">
            <line x1="2" y1="2" x2="10" y2="10" stroke="currentColor" stroke-width="1.5" />
            <line x1="10" y1="2" x2="2" y2="10" stroke="currentColor" stroke-width="1.5" />
          </svg>
        </button>
      </div>
    </div>
    <div class="mini-progress-body">
      <div v-if="error" class="mini-progress-error">{{ error }}</div>
      <template v-else>
        <div class="mini-progress-bar-row">
          <div class="mini-progress-bar">
            <div class="mini-progress-fill" :style="{ width: percent + '%' }" />
          </div>
          <span class="mini-progress-percent">{{ percent }}%</span>
        </div>
        <div v-if="progress" class="mini-progress-info">
          <span class="mini-progress-file">{{ currentFileName }}</span>
          <span class="mini-progress-bytes">
            {{ formatBytes(progress.done_bytes) }} / {{ formatBytes(progress.total_bytes) }}
          </span>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.mini-progress {
  position: fixed;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.mini-progress-titlebar {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 8px;
  background: var(--surface);
  border-bottom: 1px solid var(--border);
  -webkit-app-region: drag;
}

.mini-progress-title {
  font-size: 12px;
  font-weight: 500;
  color: var(--text);
}

.mini-progress-buttons {
  display: flex;
  gap: 4px;
  -webkit-app-region: no-drag;
}

.titlebar-btn {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  color: var(--text-muted);
  border-radius: 4px;
  cursor: pointer;
}

.titlebar-btn:hover:not(:disabled) {
  background: var(--danger);
  color: white;
}

.titlebar-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.mini-progress-body {
  position: absolute;
  top: 32px;
  left: 0;
  right: 0;
  bottom: 0;
  padding: 16px;
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: 12px;
  background: var(--surface);
}

.mini-progress-error {
  font-size: 13px;
  color: var(--danger);
}

.mini-progress-bar-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.mini-progress-bar {
  flex: 1;
  height: 8px;
  background: var(--bg);
  border-radius: 4px;
  overflow: hidden;
}

.mini-progress-fill {
  height: 100%;
  background: var(--accent);
  border-radius: 4px;
  transition: width 0.2s ease;
}

.mini-progress-percent {
  font-size: 12px;
  color: var(--text-muted);
  min-width: 36px;
  text-align: right;
}

.mini-progress-info {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
  color: var(--text-muted);
}

.mini-progress-file {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 60%;
}

.mini-progress-bytes {
  white-space: nowrap;
}
</style>
