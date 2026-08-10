import { ref } from "vue";
import {
  cancelOperation,
  createProgressChannel,
  exitApp,
  extractArchive,
} from "../api/tauri";
import { useAppStore } from "../stores/app";
import type { ChannelEvent } from "../types";

/**
 * 右键菜单「解压」流程的共享逻辑：
 * - 通过 Channel 接收进度/完成/错误事件
 * - 加密或密码错误时弹出密码框重试
 * - 出错后可取消并退出窗口（不卡死）
 */
export function useExtractFlow(archivePath: string) {
  const store = useAppStore();
  const showPassword = ref(false);
  const passwordError = ref("");

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
      passwordError.value = e.data.message;
      showPassword.value = true;
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

  function onPasswordSubmit(pw: string) {
    showPassword.value = false;
    passwordError.value = "";
    const t = pendingTarget;
    if (t) void runExtract(t, pw || undefined);
  }

  function onPasswordCancel() {
    showPassword.value = false;
    passwordError.value = "";
    pendingTarget = undefined;
    void exitApp();
  }

  function onCancel() {
    const id = store.task?.id;
    if (id != null) {
      void cancelOperation(id).catch(() => {});
    }
    void exitApp();
  }

  return {
    showPassword,
    passwordError,
    runExtract,
    onPasswordSubmit,
    onPasswordCancel,
    onCancel,
  };
}
