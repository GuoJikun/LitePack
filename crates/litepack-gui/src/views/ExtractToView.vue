<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useRoute } from "vue-router";
import { useAppStore } from "../stores/app";
import { extractArchive, createProgressChannel, exitApp } from "../api/tauri";
import type { ChannelEvent } from "../types";
import ExtractDialog from "../components/ExtractDialog.vue";
import MiniProgress from "../components/MiniProgress.vue";

const route = useRoute();
const store = useAppStore();
const archivePath = route.query.path as string;
const phase = ref<"dialog" | "progress">("dialog");
const defaultDir = ref(".");
const archiveName = ref("");

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

onMounted(() => {
  const idx = Math.max(
    archivePath.lastIndexOf("/"),
    archivePath.lastIndexOf("\\"),
  );
  defaultDir.value = idx > 0 ? archivePath.slice(0, idx) : ".";
  archiveName.value = archivePath.split(/[\\/]/).pop() ?? archivePath;
});

async function handleSubmit(dir: string) {
  phase.value = "progress";
  const progress = createProgressChannel(handleEvent);
  try {
    const id = await extractArchive({
      archive: archivePath,
      outDir: dir,
      overwrite: false,
      progress,
    });
    store.startTask(id, "extract");
  } catch {
    void exitApp();
  }
}

function handleCancel() {
  void exitApp();
}
</script>

<template>
  <ExtractDialog
    v-if="phase === 'dialog'"
    :archive-name="archiveName"
    :default-dir="defaultDir"
    @submit="handleSubmit"
    @cancel="handleCancel"
  />

  <MiniProgress
    v-if="phase === 'progress'"
    :progress="store.task?.report ?? null"
    :error="store.task?.error ?? undefined"
    @cancel="handleCancel"
  />
</template>
