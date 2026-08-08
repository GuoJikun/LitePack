<script setup lang="ts">
import { computed, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import TitleBar from "../components/TitleBar.vue";
import Toolbar from "../components/Toolbar.vue";
import EmptyState from "../components/EmptyState.vue";
import FileTable from "../components/FileTable.vue";
import StatusBar from "../components/StatusBar.vue";
import ProgressBar from "../components/ProgressBar.vue";
import PasswordDialog from "../components/PasswordDialog.vue";
import { useAppStore } from "../stores/app";
import {
  extractArchive,
  createProgressChannel,
  listArchive,
} from "../api/tauri";
import type { ChannelEvent } from "../types";

const store = useAppStore();

const loading = ref(false);
const openError = ref("");
const showPassword = ref(false);
const passwordError = ref("");
const toast = ref("");
let toastTimer: ReturnType<typeof setTimeout> | undefined;
const passwordFor = ref<"open" | "extract">("extract");
let pendingTarget: string | undefined;
let pendingOpenPath: string | undefined;
let channelReady = false;
let pendingEvents: ChannelEvent[] = [];

const canExtract = computed(() => !!store.archive && !store.task);
const hasEncrypted = computed(() => store.entries.some((e) => e.encrypted));

function showToast(msg: string) {
  toast.value = msg;
  if (toastTimer) clearTimeout(toastTimer);
  toastTimer = setTimeout(() => (toast.value = ""), 3000);
}

async function openArchive(path: string, password?: string) {
  loading.value = true;
  openError.value = "";
  try {
    const entries = await listArchive(path, password);
    const ext = (path.match(/\.([^.]+)$/)?.[1] ?? "").toLowerCase();
    const name = path.split(/[\\/]/).pop() ?? path;
    store.openArchive(path, name, ext === "7z" ? "7z" : "zip");
    store.entries = entries;
  } catch (e) {
    const msg = String(e);
    const isPw = /密码|password|加密/i.test(msg);
    if (isPw && !password) {
      pendingOpenPath = path;
      passwordFor.value = "open";
      passwordError.value = "";
      showPassword.value = true;
    } else {
      openError.value = msg;
    }
  } finally {
    loading.value = false;
  }
}

function handleEvent(e: ChannelEvent) {
  if (!channelReady) {
    pendingEvents.push(e);
    return;
  }
  dispatchEvent(e);
}

function dispatchEvent(e: ChannelEvent) {
  if (e.type === "Progress") {
    store.updateProgress(e.data);
  } else if (e.type === "Done") {
    store.finishTask(e.data.id, true);
    showToast("解压完成");
  } else if (e.type === "Error") {
    const isPw = /密码|password/i.test(e.data.message);
    if (isPw && pendingTarget) {
      store.clearTaskError();
      passwordFor.value = "extract";
      passwordError.value = e.data.message;
      showPassword.value = true;
    } else {
      store.finishTask(e.data.id, false, e.data.message);
      showToast(e.data.message);
    }
  }
}

async function runExtract(target: string, password: string | undefined) {
  if (!store.archive) return;
  channelReady = false;
  pendingEvents = [];
  const progress = createProgressChannel(handleEvent);
  try {
    const id = await extractArchive({
      archive: store.archive,
      outDir: target,
      password,
      overwrite: false,
      progress,
    });
    store.startTask(id, "extract");
    channelReady = true;
    for (const e of pendingEvents) dispatchEvent(e);
    pendingEvents = [];
  } catch (e) {
    showToast("解压失败: " + String(e));
  }
}

function startExtract(target: string, password?: string) {
  pendingTarget = target;
  if (hasEncrypted.value) {
    passwordFor.value = "extract";
    passwordError.value = "";
    showPassword.value = true;
  } else {
    void runExtract(target, password);
  }
}

async function extractTo() {
  if (!store.archive) return;
  const picked = await open({
    directory: true,
    multiple: false,
    title: "选择解压目标文件夹",
  });
  if (picked && !Array.isArray(picked)) {
    startExtract(picked);
  }
}

function extractHere() {
  if (!store.archive) return;
  const idx = Math.max(
    store.archive.lastIndexOf("/"),
    store.archive.lastIndexOf("\\"),
  );
  startExtract(idx > 0 ? store.archive.slice(0, idx) : ".");
}

function onPasswordSubmit(pw: string) {
  showPassword.value = false;
  passwordError.value = "";
  if (passwordFor.value === "open") {
    const p = pendingOpenPath;
    pendingOpenPath = undefined;
    if (p) void openArchive(p, pw || undefined);
    return;
  }
  const t = pendingTarget;
  pendingTarget = undefined;
  if (t) {
    void runExtract(t, pw || undefined);
  }
}

function onPasswordCancel() {
  showPassword.value = false;
  passwordError.value = "";
  pendingTarget = undefined;
  pendingOpenPath = undefined;
}
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

  <Transition name="toast">
    <div v-if="toast" class="toast">{{ toast }}</div>
  </Transition>

  <PasswordDialog
    v-if="showPassword"
    :error="passwordError || undefined"
    @submit="onPasswordSubmit"
    @cancel="onPasswordCancel"
  />
</template>
