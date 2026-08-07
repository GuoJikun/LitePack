<script setup lang="ts">
import { ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import FileDropZone from "./FileDropZone.vue";
import { extractArchive, createProgressChannel } from "../api/tauri";
import { useAppStore } from "../stores/app";
import type { ChannelEvent } from "../types";

const store = useAppStore();

const archives = ref<string[]>([]);
const password = ref("");
const overwrite = ref(false);
const target = ref("");
const error = ref("");

const archiveFilters = [
  { name: "压缩包", extensions: ["zip", "7z"] },
];

async function chooseTarget() {
  const picked = await open({
    directory: true,
    multiple: false,
    title: "选择解压目标文件夹",
  });
  if (picked && !Array.isArray(picked)) target.value = picked;
}

function handleEvent(e: ChannelEvent) {
  if (e.type === "Progress") {
    store.updateProgress(e.data);
  } else if (e.type === "Done") {
    store.finishTask(e.data.id, true);
  } else if (e.type === "Error") {
    store.finishTask(e.data.id, false, e.data.message);
  }
}

async function start() {
  error.value = "";
  if (!archives.value.length) {
    error.value = "请先选择压缩包";
    return;
  }
  if (!target.value) {
    error.value = "请先选择解压目标文件夹";
    return;
  }
  const progress = createProgressChannel(handleEvent);
  const id = await extractArchive({
    archive: archives.value[0],
    target: target.value,
    password: password.value || undefined,
    overwrite: overwrite.value,
    progress,
  });
  store.startTask(id, "extract");
}
</script>

<template>
  <div class="panel">
    <FileDropZone
      v-model="archives"
      :filters="archiveFilters"
      pick-title="选择要解压的压缩包"
      hint="拖拽或选择一个 ZIP / 7z 压缩包"
    />
    <div class="field">
      <label>解压到</label>
      <div class="control">
        <input type="text" v-model="target" placeholder="选择目标文件夹" />
        <button type="button" @click="chooseTarget">浏览…</button>
      </div>
    </div>
    <div class="field">
      <label>密码（加密压缩包时填写）</label>
      <input
        type="password"
        v-model="password"
        placeholder="留空表示无密码"
      />
    </div>
    <div class="field">
      <label>
        <input type="checkbox" v-model="overwrite" />
        文件已存在时覆盖
      </label>
    </div>
    <div class="row">
      <div class="grow" />
      <button type="button" class="primary" :disabled="store.active !== null" @click="start">
        开始解压
      </button>
    </div>
    <div v-if="error" class="err">{{ error }}</div>
  </div>
</template>
