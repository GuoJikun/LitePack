import { onMounted, ref } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  cancelOperation,
  createProgressChannel,
  exitApp,
  extractArchive,
  openPasswordWindow,
} from "../api/tauri";
import { useAppStore } from "../stores/app";
import type { ChannelEvent } from "../types";

/**
 * 右键菜单「解压」流程的共享逻辑：
 * - 通过 Channel 接收进度/完成/错误事件
 * - 加密或密码错误时打开独立密码窗口，通过事件回传密码
 * - 出错后可取消并退出窗口（不卡死）
 */
export function useExtractFlow(archivePath: string) {
  const store = useAppStore();
  const waitingForPassword = ref(false);

  let pendingTarget: string | undefined;
  let channelReady = false;
  let pendingEvents: ChannelEvent[] = [];

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
      return;
    }
    if (e.type === "Done") {
      store.finishTask(e.data.id, true);
      pendingTarget = undefined;
      void exitApp();
      return;
    }
    const isPw = /密码|password|加密/i.test(e.data.message);
    if (isPw && pendingTarget) {
      store.clearTaskError();
      waitingForPassword.value = true;
      void openPasswordWindow(e.data.message);
    } else {
      store.finishTask(e.data.id, false, e.data.message);
    }
  }

  async function runExtract(target: string, password?: string) {
    pendingTarget = target;
    channelReady = false;
    pendingEvents = [];
    const progress = createProgressChannel(handleEvent);
    try {
      const id = await extractArchive({
        archive: archivePath,
        outDir: target,
        password,
        overwrite: false,
        progress,
      });
      store.startTask(id, "extract");
      channelReady = true;
      for (const e of pendingEvents) dispatchEvent(e);
      pendingEvents = [];
    } catch {
      void exitApp();
    }
  }

  function onCancel() {
    const id = store.task?.id;
    if (id != null) {
      void cancelOperation(id).catch(() => {});
    }
    void exitApp();
  }

  /* 监听密码窗口回传的事件 */
  onMounted(() => {
    const win = getCurrentWindow();
    void win.listen<{ password: string }>("password-submit", (event) => {
      waitingForPassword.value = false;
      const t = pendingTarget;
      if (t) void runExtract(t, event.payload.password || undefined);
    });

    void win.listen("password-cancel", () => {
      waitingForPassword.value = false;
      pendingTarget = undefined;
      void exitApp();
    });
  });

  return {
    runExtract,
    onCancel,
  };
}
