import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// Vite 5+ 默认 target 已经支持 WebView2 (Chromium)，无需额外配置。
export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
});
