<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { useAppStore } from "../stores/app";

const store = useAppStore();

const props = defineProps<{
  onOpenArchive: (path: string) => void;
}>();

const dragging = ref(false);
let unlisten: UnlistenFn | null = null;

async function pickArchive() {
  const picked = await open({
    multiple: false,
    directory: false,
    filters: [
      { name: "压缩包", extensions: ["zip", "7z"] },
    ],
    title: "选择压缩包",
  });
  if (picked && !Array.isArray(picked)) props.onOpenArchive(picked);
}

function onDropPaths(paths: string[]) {
  const path = paths.find(
    (p) => /\.(zip|7z)$/i.test(p),
  );
  if (path) props.onOpenArchive(path);
}

onMounted(() => {
  getCurrentWebview()
    .onDragDropEvent((event) => {
      const payload = event.payload;
      if (payload.type === "enter" || payload.type === "over") {
        dragging.value = true;
      } else if (payload.type === "leave") {
        dragging.value = false;
      } else if (payload.type === "drop") {
        dragging.value = false;
        onDropPaths(payload.paths);
      }
    })
    .then((fn) => {
      unlisten = fn;
    });
});

onUnmounted(() => {
  unlisten?.();
});
</script>

<template>
  <div class="empty-state" :class="{ dragging }">
    <div class="empty-icon">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
    </div>
    <div class="empty-title">{{ dragging ? "松开以打开压缩包" : "打开压缩包" }}</div>
    <div class="empty-hint">将 ZIP / 7z 文件拖拽到此处，或点击按钮选择</div>
    <div class="empty-actions">
      <button class="empty-btn primary" @click="pickArchive">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
        打开压缩包
      </button>
      <button v-if="store.archiveName" class="empty-btn" @click="store.closeArchive()">关闭</button>
    </div>
  </div>
</template>
