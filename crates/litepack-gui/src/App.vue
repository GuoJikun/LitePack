<script setup lang="ts">
import { useAppStore } from "./stores/app";
import CompressPanel from "./components/CompressPanel.vue";
import ExtractPanel from "./components/ExtractPanel.vue";
import ArchiveList from "./components/ArchiveList.vue";
import ProgressOverlay from "./components/ProgressOverlay.vue";

const store = useAppStore();
</script>

<template>
  <header class="app-header">
    <span class="title">LitePack</span>
    <span class="hint">ZIP / 7z 压缩与解压</span>
    <div class="spacer" />
    <button type="button" @click="store.toggleTheme()">
      {{ store.theme === "light" ? "🌙" : "☀️" }}
    </button>
  </header>

  <nav class="tabs">
    <button
      type="button"
      :class="{ active: store.tab === 'compress' }"
      @click="store.tab = 'compress'"
    >
      压缩
    </button>
    <button
      type="button"
      :class="{ active: store.tab === 'extract' }"
      @click="store.tab = 'extract'"
    >
      解压
    </button>
    <button
      type="button"
      :class="{ active: store.tab === 'list' }"
      @click="store.tab = 'list'"
    >
      查看
    </button>
  </nav>

  <main class="app-main">
    <CompressPanel v-if="store.tab === 'compress'" />
    <ExtractPanel v-else-if="store.tab === 'extract'" />
    <ArchiveList v-else />
  </main>

  <ProgressOverlay />
</template>
