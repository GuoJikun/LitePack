<script setup lang="ts">
import { ref } from "vue";

const props = defineProps<{
  error?: string;
}>();

const emit = defineEmits<{
  (e: "submit", password: string): void;
  (e: "cancel"): void;
}>();

const password = ref("");
</script>

<template>
  <div class="modal-mask">
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
        <button class="modal-btn" @click="emit('cancel')">取消</button>
        <button class="modal-btn primary" @click="emit('submit', password)">解压</button>
      </div>
    </div>
  </div>
</template>
