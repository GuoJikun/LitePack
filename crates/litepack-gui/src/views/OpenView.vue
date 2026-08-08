<script setup lang="ts">
import { onMounted } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useAppStore } from "../stores/app";
import { listArchive } from "../api/tauri";

const route = useRoute();
const router = useRouter();
const store = useAppStore();
const archivePath = route.query.path as string;

onMounted(async () => {
  try {
    const entries = await listArchive(archivePath);
    const ext = (archivePath.match(/\.([^.]+)$/)?.[1] ?? "").toLowerCase();
    const name = archivePath.split(/[\\/]/).pop() ?? archivePath;
    store.openArchive(archivePath, name, ext === "7z" ? "7z" : "zip");
    store.entries = entries;
    router.replace("/");
  } catch {
    router.replace("/");
  }
});
</script>

<template>
  <div class="empty-state">
    <div class="progress-spinner" style="width: 32px; height: 32px" />
  </div>
</template>
