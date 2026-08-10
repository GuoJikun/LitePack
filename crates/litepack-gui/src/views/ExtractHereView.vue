<script setup lang="ts">
import { onMounted } from "vue";
import { useRoute } from "vue-router";
import { useExtractFlow } from "../composables/useExtractFlow";
import { useAppStore } from "../stores/app";
import MiniProgress from "../components/MiniProgress.vue";
import PasswordDialog from "../components/PasswordDialog.vue";

const store = useAppStore();
const route = useRoute();
const archivePath = route.query.path as string;

const { showPassword, passwordError, runExtract, onPasswordSubmit, onPasswordCancel, onCancel } =
  useExtractFlow(archivePath);

onMounted(() => {
  const idx = Math.max(
    archivePath.lastIndexOf("/"),
    archivePath.lastIndexOf("\\"),
  );
  const outDir = idx > 0 ? archivePath.slice(0, idx) : ".";
  void runExtract(outDir);
});
</script>

<template>
  <MiniProgress
    :progress="store.task?.report ?? null"
    :error="store.task?.error ?? undefined"
    @cancel="onCancel"
  />

  <PasswordDialog
    v-if="showPassword"
    :error="passwordError || undefined"
    @submit="onPasswordSubmit"
    @cancel="onPasswordCancel"
  />
</template>
