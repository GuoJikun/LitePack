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
  target: string;
  password?: string;
  overwrite: boolean;
  progress: Channel<ChannelEvent>;
}

export function extractArchive(req: ExtractRequest): Promise<number> {
  return invoke("extract_archive", {
    archive: req.archive,
    target: req.target,
    password: req.password,
    overwrite: req.overwrite,
    progress: req.progress,
  });
}

export function listArchive(archive: string): Promise<EntryInfo[]> {
  return invoke("list_archive", { archive });
}

export function cancelOperation(id: number): Promise<void> {
  return invoke("cancel_operation", { id });
}
