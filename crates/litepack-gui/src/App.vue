<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import TitleBar from "./components/TitleBar.vue";
import Toolbar from "./components/Toolbar.vue";
import EmptyState from "./components/EmptyState.vue";
import FileTable from "./components/FileTable.vue";
import StatusBar from "./components/StatusBar.vue";
import ProgressBar from "./components/ProgressBar.vue";
import PasswordDialog from "./components/PasswordDialog.vue";
import { useAppStore } from "./stores/app";
import {
  extractArchive,
  createProgressChannel,
  listArchive,
  takePendingExtract,
  exitApp,
} from "./api/tauri";
import type { ChannelEvent } from "./types";

const store = useAppStore();

const loading = ref(false);
const openError = ref("");
const showPassword = ref(false);
const passwordError = ref("");
let pendingTarget: string | undefined;
let autoExit = false;

const canExtract = computed(() => !!store.archive && !store.task);

async function openArchive(path: string) {
  loading.value = true;
  openError.value = "";
  try {
    const entries = await listArchive(path);
    const ext = (path.match(/\.([^.]+)$/)?.[1] ?? "").toLowerCase();
    const name = path.split(/[\\/]/).pop() ?? path;
    store.openArchive(path, name, ext === "7z" ? "7z" : "zip");
    store.entries = entries;
  } catch (e) {
    openError.value = String(e);
  } finally {
    loading.value = false;
  }
}

function handleEvent(e: ChannelEvent) {
  if (e.type === "Progress") {
    store.updateProgress(e.data);
  } else if (e.type === "Done") {
    store.finishTask(e.data.id, true);
    if (autoExit) {
      void exitApp();
    }
  } else if (e.type === "Error") {
    const isPw = /密码|password/i.test(e.data.message);
    if (isPw && pendingTarget) {
      // 需要密码（或密码错误），弹出输入框
      store.clearTaskError();
      passwordError.value = e.data.message;
      showPassword.value = true;
    } else {
      store.finishTask(e.data.id, false, e.data.message);
    }
  }
}

async function runExtract(target: string, password: string | undefined) {
  if (!store.archive) return;
  const progress = createProgressChannel(handleEvent);
  const id = await extractArchive({
    archive: store.archive,
    outDir: target,
    password,
    overwrite: false,
    progress,
  });
  store.startTask(id, "extract");
}

async function extractTo() {
  if (!store.archive) return;
  const picked = await open({
    directory: true,
    multiple: false,
    title: "选择解压目标文件夹",
  });
  if (picked && !Array.isArray(picked)) {
    pendingTarget = picked;
    showPassword.value = true;
    passwordError.value = "";
  }
}

function extractHere() {
  if (!store.archive) return;
  const idx = Math.max(
    store.archive.lastIndexOf("/"),
    store.archive.lastIndexOf("\\"),
  );
  pendingTarget = idx > 0 ? store.archive.slice(0, idx) : ".";
  showPassword.value = true;
  passwordError.value = "";
}

function onPasswordSubmit(pw: string) {
  showPassword.value = false;
  passwordError.value = "";
  if (pendingTarget) {
    void runExtract(pendingTarget, pw || undefined);
  }
}

function onPasswordCancel() {
  showPassword.value = false;
  passwordError.value = "";
  pendingTarget = undefined;
}

// 右键菜单「直接解压」入口：启动时携带 --extract-here <path>，
// 自动解压到归档所在目录，完成后退出应用。
async function handlePendingExtract(path: string) {
  await openArchive(path);
  if (!store.archive) return;
  autoExit = true;
  const idx = Math.max(
    store.archive.lastIndexOf("/"),
    store.archive.lastIndexOf("\\"),
  );
  pendingTarget = idx > 0 ? store.archive.slice(0, idx) : ".";
  await runExtract(pendingTarget, undefined);
}

onMounted(async () => {
  const pending = await takePendingExtract();
  if (pending) {
    await handlePendingExtract(pending);
  }
});
</script>

<template>
  <TitleBar />
  <Toolbar
    :can-extract="canExtract"
    :on-extract-to="extractTo"
    :on-extract-here="extractHere"
  />

  <div v-if="store.archive" class="content">
    <div v-if="loading" class="empty-state">
      <div class="progress-spinner" style="width: 32px; height: 32px" />
    </div>
    <template v-else>
      <FileTable />
      <StatusBar />
    </template>
  </div>

  <EmptyState v-else :on-open-archive="openArchive" />

  <div
    v-if="openError"
    class="statusbar"
    style="border-top: 1px solid var(--danger-soft); color: var(--danger)"
  >
    <span class="status-item">打开失败: {{ openError }}</span>
  </div>

  <ProgressBar />

  <PasswordDialog
    v-if="showPassword"
    :error="passwordError || undefined"
    @submit="onPasswordSubmit"
    @cancel="onPasswordCancel"
  />
</template>
