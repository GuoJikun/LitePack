<script setup lang="ts">
import { ref, computed } from "vue";
import { useRoute } from "vue-router";
import { getCurrentWindow } from "@tauri-apps/api/window";

const route = useRoute();
const win = getCurrentWindow();
const password = ref("");

const errorMessage = computed(() => {
  const raw = route.query.error;
  return typeof raw === "string" ? decodeURIComponent(raw) : "";
});

async function onSubmit() {
  await win.emit("password-submit", { password: password.value });
  void win.close();
}

async function onCancel() {
  await win.emit("password-cancel");
  void win.close();
}
</script>

<template>
  <div class="password-window">
    <div class="password-titlebar">
      <span class="password-titlebar-text">LitePack - 输入密码</span>
      <div class="password-titlebar-buttons">
        <button class="titlebar-btn" @click="onCancel">
          <svg viewBox="0 0 12 12" width="12" height="12">
            <line x1="2" y1="2" x2="10" y2="10" stroke="currentColor" stroke-width="1.5" />
            <line x1="10" y1="2" x2="2" y2="10" stroke="currentColor" stroke-width="1.5" />
          </svg>
        </button>
      </div>
    </div>
    <div class="password-body">
      <div v-if="errorMessage" class="password-window-error">{{ errorMessage }}</div>
      <input
        v-model="password"
        class="modal-input"
        type="password"
        placeholder="请输入密码"
        autofocus
        @keydown.enter="onSubmit"
      />
      <div class="modal-actions">
        <button class="modal-btn" @click="onCancel">取消</button>
        <button class="modal-btn primary" @click="onSubmit">解压</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.password-window {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: var(--surface);
}

.password-titlebar {
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 8px;
  background: var(--surface);
  border-bottom: 1px solid var(--border);
  -webkit-app-region: drag;
  flex-shrink: 0;
}

.password-titlebar-text {
  font-size: 12px;
  font-weight: 500;
  color: var(--text);
}

.password-titlebar-buttons {
  display: flex;
  gap: 4px;
  -webkit-app-region: no-drag;
}

.titlebar-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  border-radius: 4px;
  cursor: pointer;
  padding: 0;
}

.titlebar-btn:hover {
  background: var(--hover);
  color: var(--text);
}

.password-body {
  flex: 1;
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.password-window-error {
  color: var(--danger);
  font-size: 12px;
  margin-bottom: 10px;
}
</style>
