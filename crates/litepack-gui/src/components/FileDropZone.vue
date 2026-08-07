<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import type { UnlistenFn } from "@tauri-apps/api/event";

const props = withDefaults(
  defineProps<{
    modelValue: string[];
    pickTitle: string;
    filters?: { name: string; extensions: string[] }[];
    hint: string;
  }>(),
  { filters: undefined, hint: "" },
);

const emit = defineEmits<{ (e: "update:modelValue", v: string[]): void }>();

const dragging = ref(false);
let unlisten: UnlistenFn | null = null;

function addPaths(paths: string[]) {
  const seen = new Set(props.modelValue);
  const next = [...props.modelValue];
  for (const p of paths) {
    if (!seen.has(p)) {
      seen.add(p);
      next.push(p);
    }
  }
  emit("update:modelValue", next);
}

async function pickFiles() {
  const picked = await open({
    multiple: true,
    directory: false,
    filters: props.filters,
    title: props.pickTitle,
  });
  if (!picked) return;
  addPaths(Array.isArray(picked) ? picked : [picked]);
}

async function pickFolder() {
  const picked = await open({
    multiple: true,
    directory: true,
    title: props.pickTitle,
  });
  if (!picked) return;
  addPaths(Array.isArray(picked) ? picked : [picked]);
}

function removePath(p: string) {
  emit(
    "update:modelValue",
    props.modelValue.filter((x) => x !== p),
  );
}

onMounted(() => {
  getCurrentWebview()
    .onDragDropEvent((event) => {
      const payload = event.payload;
      if (payload.type === "enter" || payload.type === "over") {
        dragging.value = true;
      } else if (payload.type === "leave") {
        dragging.value = false;
      } else if (payload.type === "drop") {
        dragging.value = false;
        addPaths(payload.paths);
      }
    })
    .then((fn) => {
      unlisten = fn;
    });
});

onUnmounted(() => {
  unlisten?.();
});
</script>

<template>
  <div>
    <div class="drop-zone" :class="{ dragging }">
      <div>{{ hint || "将文件或文件夹拖拽到此处" }}</div>
      <div class="actions">
        <button type="button" @click="pickFiles">选择文件</button>
        <button type="button" @click="pickFolder">选择文件夹</button>
      </div>
    </div>
    <div v-if="modelValue.length" class="file-list">
      <div v-for="p in modelValue" :key="p" class="item">
        <span class="name" :title="p">{{ p }}</span>
        <button type="button" @click="removePath(p)">✕</button>
      </div>
    </div>
  </div>
</template>
