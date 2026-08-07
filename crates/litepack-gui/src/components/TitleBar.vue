<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useAppStore } from "../stores/app";
import { humanSize } from "../utils/format";

const store = useAppStore();
const win = getCurrentWindow();

function minimize() {
  void win.minimize();
}

async function toggleMaximize() {
  if (await win.isMaximized()) void win.unmaximize();
  else void win.maximize();
}

function close() {
  void win.close();
}
</script>

<template>
  <div class="titlebar">
    <div class="titlebar-brand">
      <div class="titlebar-logo">L</div>
      <span class="titlebar-name">LitePack</span>
    </div>
    <div v-if="store.archiveName" class="titlebar-archive" :title="store.archiveName">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
      {{ store.archiveName }}
    </div>
    <div class="titlebar-spacer" @dblclick="toggleMaximize" />
    <span v-if="store.archive" class="titlebar-info">
      解包大小 {{ humanSize(store.dirTotalSize) }}
    </span>
    <div class="titlebar-actions">
      <button title="最小化" @click="minimize">─</button>
      <button title="最大化" @click="toggleMaximize">☐</button>
      <button class="close-btn" title="关闭" @click="close">✕</button>
    </div>
  </div>
</template>
