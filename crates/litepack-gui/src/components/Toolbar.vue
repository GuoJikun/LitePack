<script setup lang="ts">
import { useAppStore } from "../stores/app";
import SettingsMenu from "./SettingsMenu.vue";

const store = useAppStore();

defineProps<{
  canExtract: boolean;
  onExtractTo: () => void;
  onExtractHere: () => void;
}>();
</script>

<template>
  <div class="toolbar">
    <button class="tool-btn" title="解压到指定文件夹" :disabled="!canExtract" @click="onExtractTo">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
      <span>解压到</span>
    </button>
    <button class="tool-btn" title="一键解压到当前目录" :disabled="!canExtract" @click="onExtractHere">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg>
      <span>一键解压</span>
    </button>
    <div class="tool-right">
      <div v-if="canExtract" class="tool-search">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
        <input type="text" v-model="store.search" placeholder="搜索包内文件…" />
      </div>
      <button class="theme-toggle" title="切换主题" @click="store.toggleTheme()">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <template v-if="store.theme === 'dark'">
            <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
          </template>
          <template v-else>
            <circle cx="12" cy="12" r="5" />
            <line x1="12" y1="1" x2="12" y2="3" />
            <line x1="12" y1="21" x2="12" y2="23" />
            <line x1="4.22" y1="4.22" x2="5.64" y2="5.64" />
            <line x1="18.36" y1="18.36" x2="19.78" y2="19.78" />
            <line x1="1" y1="12" x2="3" y2="12" />
            <line x1="21" y1="12" x2="23" y2="12" />
            <line x1="4.22" y1="19.78" x2="5.64" y2="18.36" />
            <line x1="18.36" y1="5.64" x2="19.78" y2="4.22" />
          </template>
        </svg>
      </button>
      <SettingsMenu />
    </div>
  </div>
</template>
