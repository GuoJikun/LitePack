<script setup lang="ts">
import { ref } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import FileDropZone from "./FileDropZone.vue";
import OptionBar from "./OptionBar.vue";
import { compressFiles, createProgressChannel } from "../api/tauri";
import { useAppStore } from "../stores/app";
import type { ArchiveFormat, ChannelEvent } from "../types";

const store = useAppStore();

const files = ref<string[]>([]);
const format = ref<ArchiveFormat>("zip");
const level = ref(6);
const password = ref("");
const skipHidden = ref(true);
const overwrite = ref(false);
const target = ref("");
const error = ref("");

const filters = [
  { name: "压缩包", extensions: ["zip", "7z"] },
];

async function chooseTarget() {
  const picked = await save({
    title: "保存压缩包",
    defaultPath: `archive.${format.value}`,
    filters,
  });
  if (picked) target.value = picked;
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
  if (!files.value.length) {
    error.value = "请先选择要压缩的文件或文件夹";
    return;
  }
  if (!target.value) {
    error.value = "请先选择压缩包保存位置";
    return;
  }
  const progress = createProgressChannel(handleEvent);
  const id = await compressFiles({
    paths: files.value,
    target: target.value,
    format: format.value,
    level: level.value,
    password: password.value || undefined,
    skipHidden: skipHidden.value,
    overwrite: overwrite.value,
    progress,
  });
  store.startTask(id, "compress");
}
</script>

<template>
  <div class="panel">
    <FileDropZone
      v-model="files"
      pick-title="选择要压缩的文件或文件夹"
      hint="添加文件 / 文件夹后开始压缩"
    />
    <OptionBar
      v-model:format="format"
      v-model:level="level"
      v-model:password="password"
      v-model:skip-hidden="skipHidden"
    />
    <div class="field">
      <label>保存到</label>
      <div class="control">
        <input type="text" v-model="target" placeholder="选择输出压缩包路径" />
        <button type="button" @click="chooseTarget">浏览…</button>
      </div>
    </div>
    <div class="field">
      <label>
        <input type="checkbox" v-model="overwrite" />
        目标已存在时覆盖
      </label>
    </div>
    <div class="row">
      <div class="grow" />
      <button type="button" class="primary" :disabled="store.active !== null" @click="start">
        开始压缩
      </button>
    </div>
    <div v-if="error" class="err">{{ error }}</div>
  </div>
</template>
