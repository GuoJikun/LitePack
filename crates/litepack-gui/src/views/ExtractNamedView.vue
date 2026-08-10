<script setup lang="ts">
import { onMounted } from "vue";
import { useRoute } from "vue-router";
import { useExtractFlow } from "../composables/useExtractFlow";
import { useAppStore } from "../stores/app";
import MiniProgress from "../components/MiniProgress.vue";

const store = useAppStore();
const route = useRoute();
const archivePath = route.query.path as string;

const { runExtract, onCancel } = useExtractFlow(archivePath);

onMounted(() => {
  const idx = Math.max(
    archivePath.lastIndexOf("/"),
    archivePath.lastIndexOf("\\"),
  );
  const dir = idx > 0 ? archivePath.slice(0, idx) : ".";
  const baseName = archivePath.replace(/\.[^.]+$/, "").split(/[\\/]/).pop() ?? "output";
  const outDir = `${dir}/${baseName}`;
  void runExtract(outDir);
});
</script>

<template>
  <MiniProgress
    :progress="store.task?.report ?? null"
    :error="store.task?.error ?? undefined"
    @cancel="onCancel"
  />
</template>
