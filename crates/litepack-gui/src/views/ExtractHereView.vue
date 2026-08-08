<script setup lang="ts">
import { onMounted } from "vue";
import { useRoute } from "vue-router";
import { useAppStore } from "../stores/app";
import { extractArchive, createProgressChannel, exitApp } from "../api/tauri";
import type { ChannelEvent } from "../types";
import MiniProgress from "../components/MiniProgress.vue";

const route = useRoute();
const store = useAppStore();
const archivePath = route.query.path as string;

function handleEvent(e: ChannelEvent) {
  if (e.type === "Progress") {
    store.updateProgress(e.data);
  } else if (e.type === "Done") {
    store.finishTask(e.data.id, true);
    void exitApp();
  } else if (e.type === "Error") {
    store.finishTask(e.data.id, false, e.data.message);
  }
}

onMounted(async () => {
  const idx = Math.max(
    archivePath.lastIndexOf("/"),
    archivePath.lastIndexOf("\\"),
  );
  const outDir = idx > 0 ? archivePath.slice(0, idx) : ".";

  const progress = createProgressChannel(handleEvent);
  try {
    const id = await extractArchive({
      archive: archivePath,
      outDir,
      overwrite: false,
      progress,
    });
    store.startTask(id, "extract");
  } catch {
    void exitApp();
  }
});
</script>

<template>
  <MiniProgress
    :progress="store.task?.report ?? null"
    :error="store.task?.error ?? undefined"
    @cancel="() => exitApp()"
  />
</template>
