<script setup lang="ts">
import { ref } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";

const props = defineProps<{
  error?: string;
}>();

const emit = defineEmits<{
  (e: "submit", password: string): void;
  (e: "cancel"): void;
}>();

const password = ref("");

function closeWindow() {
  getCurrentWindow().close().catch(() => {});
}
</script>

<template>
  <div class="modal-mask" role="dialog" aria-modal="true" aria-label="输入压缩包密码">
    <div class="modal">
      <div class="modal-title">该压缩包已加密</div>
      <div v-if="props.error" class="modal-err">{{ props.error }}</div>
      <input
        class="modal-input"
        type="password"
        v-model="password"
        placeholder="请输入密码"
        @keydown.enter="emit('submit', password)"
        autofocus
      />
      <div class="modal-actions">
        <button class="modal-btn" @click="emit('cancel'); closeWindow()">取消</button>
        <button class="modal-btn primary" @click="emit('submit', password); closeWindow()">解压</button>
      </div>
    </div>
  </div>
</template>
