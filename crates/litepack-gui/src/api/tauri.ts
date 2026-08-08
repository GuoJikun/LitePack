import { invoke, Channel } from "@tauri-apps/api/core";
import type { ArchiveFormat, ChannelEvent, EntryInfo } from "../types";

export function createProgressChannel(
  onEvent: (e: ChannelEvent) => void,
): Channel<ChannelEvent> {
  return new Channel(onEvent);
}

export interface CompressRequest {
  paths: string[];
  target: string;
  format: ArchiveFormat;
  level: number;
  password?: string;
  skipHidden: boolean;
  overwrite: boolean;
  progress: Channel<ChannelEvent>;
}

export function compressFiles(req: CompressRequest): Promise<number> {
  return invoke("compress_files", {
    paths: req.paths,
    target: req.target,
    format: req.format,
    level: req.level,
    password: req.password,
    skipHidden: req.skipHidden,
    overwrite: req.overwrite,
    progress: req.progress,
  });
}

export interface ExtractRequest {
  archive: string;
  outDir: string;
  password?: string;
  overwrite: boolean;
  progress: Channel<ChannelEvent>;
}

export function extractArchive(req: ExtractRequest): Promise<number> {
  return invoke("extract_archive", {
    archive: req.archive,
    out_dir: req.outDir,
    password: req.password,
    overwrite: req.overwrite,
    progress: req.progress,
  });
}

export function listArchive(archive: string, password?: string): Promise<EntryInfo[]> {
  return invoke("list_archive", { archive, password });
}

export function cancelOperation(id: number): Promise<void> {
  return invoke("cancel_operation", { id });
}

export function registerContextMenu(): Promise<void> {
  return invoke("register_context_menu");
}

export function unregisterContextMenu(): Promise<void> {
  return invoke("unregister_context_menu");
}

export function contextMenuStatus(): Promise<boolean> {
  return invoke("context_menu_status");
}

export function takePendingExtract(): Promise<string | null> {
  return invoke("take_pending_extract");
}

export function exitApp(): Promise<void> {
  return invoke("exit_app");
}
