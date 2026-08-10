<script setup lang="ts">
import { computed } from "vue";
import { useRoute } from "vue-router";
import { getCurrentWindow } from "@tauri-apps/api/window";

const route = useRoute();
const errorMessage = computed(() => {
  const raw = route.query.error;
  return typeof raw === "string" ? decodeURIComponent(raw) : "";
});

function onSubmit(password: string) {
  // 这里先用一个简单的占位：关闭子窗口，后续再接入主窗口回传逻辑。
  getCurrentWindow().close().catch(() => {});
}
</script>

<template>
  <div class="password-window">
    <div class="password-window-title">请输入密码</div>
    <div v-if="errorMessage" class="password-window-error">{{ errorMessage }}</div>
    <input class="modal-input" type="password" placeholder="请输入密码" />
    <div class="modal-actions">
      <button class="modal-btn" @click="getCurrentWindow().close().catch(() => {})">取消</button>
      <button class="modal-btn primary" @click="onSubmit('')">解压</button>
    </div>
  </div>
</template>

<style scoped>
.password-window {
  height: 100vh;
  padding: 20px;
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  justify-content: center;
  background: var(--surface);
}

.password-window-title {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 8px;
}

.password-window-error {
  color: var(--danger);
  font-size: 12px;
  margin-bottom: 10px;
}
</style>
