<script setup lang="ts">
import { ref, onMounted } from "vue";
import { open } from "@tauri-apps/plugin-dialog";

const props = defineProps<{
  archiveName: string;
  defaultDir: string;
}>();

const emit = defineEmits<{
  (e: "submit", dir: string): void;
  (e: "cancel"): void;
}>();

const targetDir = ref(props.defaultDir);

async function pickDir() {
  const picked = await open({
    directory: true,
    multiple: false,
    title: "选择解压目标文件夹",
    defaultPath: targetDir.value,
  });
  if (picked && !Array.isArray(picked)) {
    targetDir.value = picked;
  }
}

function handleSubmit() {
  if (targetDir.value.trim()) {
    emit("submit", targetDir.value);
  }
}
</script>

<template>
  <div class="extract-dialog">
    <div class="extract-dialog-titlebar">
      <span class="extract-dialog-title">解压文件</span>
      <div class="extract-dialog-buttons">
        <button class="titlebar-btn" @click="emit('cancel')">
          <svg viewBox="0 0 12 12" width="12" height="12">
            <line x1="2" y1="2" x2="10" y2="10" stroke="currentColor" stroke-width="1.5" />
            <line x1="10" y1="2" x2="2" y2="10" stroke="currentColor" stroke-width="1.5" />
          </svg>
        </button>
      </div>
    </div>
    <div class="extract-dialog-body">
      <label class="extract-label">目标路径（如果不存在将被创建）：</label>
      <div class="extract-path-row">
        <input
          class="extract-path-input"
          v-model="targetDir"
          @keydown.enter="handleSubmit"
          autofocus
        />
        <button class="extract-pick-btn" @click="pickDir" title="选择文件夹">
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
          </svg>
        </button>
      </div>
    </div>
    <div class="extract-dialog-footer">
      <span class="extract-advanced">▶ 高级选项</span>
      <button class="extract-submit-btn" @click="handleSubmit">立即解压</button>
    </div>
  </div>
</template>

<style scoped>
.extract-dialog {
  position: fixed;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.extract-dialog-titlebar {
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

.extract-dialog-title {
  font-size: 12px;
  font-weight: 500;
  color: var(--text);
}

.extract-dialog-buttons {
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

.titlebar-btn:hover {
  background: var(--danger);
  color: white;
}

.extract-dialog-body {
  position: absolute;
  top: 32px;
  left: 0;
  right: 0;
  bottom: 48px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  background: var(--surface);
}

.extract-label {
  font-size: 12px;
  color: var(--text-muted);
}

.extract-path-row {
  display: flex;
  gap: 8px;
}

.extract-path-input {
  flex: 1;
  height: 32px;
  padding: 0 8px;
  border: 1px solid var(--border);
  border-radius: 4px;
  background: var(--bg);
  color: var(--text);
  font-size: 13px;
  outline: none;
}

.extract-path-input:focus {
  border-color: var(--accent);
}

.extract-pick-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--border);
  border-radius: 4px;
  background: var(--bg);
  color: var(--text-muted);
  cursor: pointer;
}

.extract-pick-btn:hover {
  background: var(--surface);
  color: var(--text);
}

.extract-dialog-footer {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  background: var(--surface);
  border-top: 1px solid var(--border);
}

.extract-advanced {
  font-size: 12px;
  color: var(--text-muted);
  cursor: pointer;
}

.extract-advanced:hover {
  color: var(--text);
}

.extract-submit-btn {
  height: 28px;
  padding: 0 16px;
  border: none;
  border-radius: 4px;
  background: var(--accent);
  color: white;
  font-size: 13px;
  cursor: pointer;
}

.extract-submit-btn:hover {
  opacity: 0.9;
}
</style>
