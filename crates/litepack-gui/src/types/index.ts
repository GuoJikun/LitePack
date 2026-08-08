export type ArchiveFormat = "zip" | "7z";
export type Phase = "Listing" | "Compressing" | "Extracting";

export interface EntryInfo {
  path: string;
  size: number;
  is_dir: boolean;
  compressed_size: number | null;
  method: string | null;
  modified: number | null;
  encrypted: boolean;
}

export interface ProgressReport {
  phase: Phase;
  current_file: string;
  done_bytes: number;
  total_bytes: number;
}

export type ChannelEvent =
  | { type: "Progress"; data: ProgressReport }
  | { type: "Done"; data: { id: number } }
  | { type: "Error"; data: { id: number; message: string } };
