<script setup lang="ts">
import { computed } from "vue";
import { useAppStore } from "../stores/app";
import { humanSize, ratioPercent } from "../utils/format";

const store = useAppStore();

const ratio = computed(() =>
  ratioPercent(store.dirTotalSize, store.dirTotalCompressed),
);
</script>

<template>
  <div class="statusbar">
    <span class="status-item">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
      大小: {{ humanSize(store.dirTotalSize) }}
    </span>
    <span class="status-item">共 {{ store.children.length }} 个{{ store.currentDir ? "条目" : "文件" }}</span>
    <span v-if="ratio != null" class="status-item">压缩率: <span class="ratio">{{ ratio }}%</span></span>
    <span class="spacer" />
    <span class="status-item">{{ store.archiveFormat.toUpperCase() }} 格式</span>
  </div>
</template>
