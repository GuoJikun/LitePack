<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useRoute } from "vue-router";
import { useExtractFlow } from "../composables/useExtractFlow";
import { useAppStore } from "../stores/app";
import ExtractDialog from "../components/ExtractDialog.vue";
import MiniProgress from "../components/MiniProgress.vue";
import PasswordDialog from "../components/PasswordDialog.vue";

const store = useAppStore();
const route = useRoute();
const archivePath = route.query.path as string;
const phase = ref<"dialog" | "progress">("dialog");
const defaultDir = ref(".");
const archiveName = ref("");

const { showPassword, passwordError, runExtract, onPasswordSubmit, onPasswordCancel, onCancel } =
  useExtractFlow(archivePath);

onMounted(() => {
  const idx = Math.max(
    archivePath.lastIndexOf("/"),
    archivePath.lastIndexOf("\\"),
  );
  defaultDir.value = idx > 0 ? archivePath.slice(0, idx) : ".";
  archiveName.value = archivePath.split(/[\\/]/).pop() ?? archivePath;
});

function onDialogSubmit(dir: string) {
  phase.value = "progress";
  void runExtract(dir);
}

function handleCancel() {
  void onCancel();
}
</script>

<template>
  <ExtractDialog
    v-if="phase === 'dialog'"
    :archive-name="archiveName"
    :default-dir="defaultDir"
    @submit="onDialogSubmit"
    @cancel="handleCancel"
  />

  <MiniProgress
    v-if="phase === 'progress'"
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
