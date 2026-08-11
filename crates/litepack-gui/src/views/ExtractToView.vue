<script setup lang="ts">
import { ref } from "vue";
import { useRoute } from "vue-router";
import { useExtractFlow } from "../composables/useExtractFlow";
import { useAppStore } from "../stores/app";
import ExtractDialog from "../components/ExtractDialog.vue";
import MiniProgress from "../components/MiniProgress.vue";

const store = useAppStore();
const route = useRoute();
const archivePath = route.query.path as string;
const phase = ref<"dialog" | "progress">("dialog");

// 同步计算，确保首次渲染即为正确值（onMounted 会导致子组件 ref 拿到初始 "."）
const idx = Math.max(
  archivePath.lastIndexOf("/"),
  archivePath.lastIndexOf("\\"),
);
const defaultDir = idx > 0 ? archivePath.slice(0, idx) : ".";
const archiveName = archivePath.split(/[\\/]/).pop() ?? archivePath;

const { runExtract, onCancel } = useExtractFlow(archivePath);

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
</template>
