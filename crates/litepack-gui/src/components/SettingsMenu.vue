<script setup lang="ts">
import { onMounted, ref } from "vue";
import {
  contextMenuStatus,
  registerContextMenu,
  unregisterContextMenu,
} from "../api/tauri";

const open = ref(false);
const contextMenu = ref(false);
const busy = ref(false);
const error = ref("");

onMounted(async () => {
  try {
    contextMenu.value = await contextMenuStatus();
  } catch {
    contextMenu.value = false;
  }
});

async function toggleContextMenu() {
  if (busy.value) return;
  busy.value = true;
  error.value = "";
  const target = !contextMenu.value;
  try {
    if (target) {
      await registerContextMenu();
    } else {
      await unregisterContextMenu();
    }
    contextMenu.value = target;
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="settings-menu">
    <button class="theme-toggle" title="设置" @click="open = !open">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="3" />
        <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
      </svg>
    </button>

    <div v-if="open" class="settings-popover" @click.stop>
      <div class="settings-title">设置</div>
      <label class="settings-row">
        <span class="settings-label">资源管理器右键菜单「直接解压」</span>
        <span class="switch" :class="{ on: contextMenu }" :aria-disabled="busy" @click="toggleContextMenu">
          <span class="switch-knob" />
        </span>
      </label>
      <div v-if="error" class="settings-error">{{ error }}</div>
      <div v-if="busy" class="settings-busy">处理中…</div>
    </div>

    <div v-if="open" class="settings-backdrop" @click="open = false" />
  </div>
</template>

<style scoped>
.settings-menu {
  position: relative;
}

.settings-popover {
  position: absolute;
  top: 36px;
  right: 0;
  width: 300px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-md);
  padding: 12px;
  z-index: 50;
}

.settings-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-2);
  margin-bottom: 10px;
}

.settings-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  cursor: pointer;
}

.settings-label {
  font-size: 12px;
  line-height: 1.45;
  color: var(--text);
}

.switch {
  width: 36px;
  height: 20px;
  border-radius: 10px;
  background: var(--border);
  position: relative;
  flex-shrink: 0;
  transition: background var(--t);
}

.switch.on {
  background: var(--accent);
}

.switch-knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: #fff;
  transition: left var(--t);
}

.switch.on .switch-knob {
  left: 18px;
}

.settings-error {
  margin-top: 8px;
  font-size: 11px;
  color: var(--danger);
}

.settings-busy {
  margin-top: 8px;
  font-size: 11px;
  color: var(--text-3);
}

.settings-backdrop {
  position: fixed;
  inset: 0;
  z-index: 40;
}
</style>
