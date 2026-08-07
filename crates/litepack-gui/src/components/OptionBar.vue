<script setup lang="ts">
import type { ArchiveFormat } from "../types";

const props = withDefaults(
  defineProps<{
    format: ArchiveFormat;
    level: number;
    password?: string;
    skipHidden: boolean;
  }>(),
  { password: "" },
);

const emit = defineEmits<{
  (e: "update:format", v: ArchiveFormat): void;
  (e: "update:level", v: number): void;
  (e: "update:password", v: string): void;
  (e: "update:skipHidden", v: boolean): void;
}>();
</script>

<template>
  <div>
    <div class="field">
      <label>压缩格式</label>
      <div class="radio-pills">
        <button
          type="button"
          :class="{ active: props.format === 'zip' }"
          @click="emit('update:format', 'zip')"
        >
          ZIP
        </button>
        <button
          type="button"
          :class="{ active: props.format === '7z' }"
          @click="emit('update:format', '7z')"
        >
          7z
        </button>
      </div>
    </div>
    <div class="field">
      <label>压缩级别：{{ props.level }}</label>
      <input
        type="range"
        min="0"
        max="9"
        :value="props.level"
        @input="emit('update:level', Number(($event.target as HTMLInputElement).value))"
      />
    </div>
    <div class="field">
      <label>密码（可选）</label>
      <input
        type="password"
        :value="props.password"
        placeholder="留空表示不加密"
        @input="emit('update:password', ($event.target as HTMLInputElement).value)"
      />
    </div>
    <div class="field">
      <label>
        <input
          type="checkbox"
          :checked="props.skipHidden"
          @change="emit('update:skipHidden', ($event.target as HTMLInputElement).checked)"
        />
        跳过隐藏文件 / 文件夹
      </label>
    </div>
  </div>
</template>
