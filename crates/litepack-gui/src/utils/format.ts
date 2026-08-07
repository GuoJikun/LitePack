import type { EntryInfo } from "../types";

export type FileIconKind = "folder" | "zip" | "doc" | "img" | "code";

const IMG_EXT = ["png", "jpg", "jpeg", "gif", "svg", "webp", "bmp", "ico", "avif"];
const DOC_EXT = [
  "txt", "md", "pdf", "doc", "docx", "epub", "mobi", "azw3", "rtf",
  "xls", "xlsx", "ppt", "pptx", "csv",
];
const ZIP_EXT = ["zip", "7z", "rar", "gz", "bz2", "xz", "tar", "lz4", "zst"];
const CODE_EXT = [
  "rs", "js", "ts", "tsx", "jsx", "json", "html", "htm", "css", "scss",
  "vue", "py", "c", "cpp", "h", "hpp", "go", "java", "kt", "swift",
  "sh", "bat", "ps1", "yaml", "yml", "toml", "xml", "sql", "ini", "conf",
];

export function iconKind(e: EntryInfo): FileIconKind {
  if (e.is_dir) return "folder";
  const dot = e.path.lastIndexOf(".");
  const ext = dot >= 0 ? e.path.slice(dot + 1).toLowerCase() : "";
  if (ZIP_EXT.includes(ext)) return "zip";
  if (IMG_EXT.includes(ext)) return "img";
  if (DOC_EXT.includes(ext)) return "doc";
  if (CODE_EXT.includes(ext)) return "code";
  return "code";
}

export function typeLabel(e: EntryInfo): string {
  if (e.is_dir) return "文件夹";
  const dot = e.path.lastIndexOf(".");
  const ext = dot >= 0 ? e.path.slice(dot + 1).toLowerCase() : "";
  if (!ext) return "文件";
  return `${ext.toUpperCase()} 文件`;
}

export function humanSize(n: number | null): string {
  if (n == null) return "—";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let v = n;
  let i = 0;
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024;
    i++;
  }
  const digits = i === 0 || v >= 100 ? 0 : v >= 10 ? 1 : 2;
  return `${v.toFixed(digits)} ${units[i]}`;
}

export function formatDate(unix: number | null): string {
  if (unix == null) return "—";
  const d = new Date(unix * 1000);
  const pad = (x: number) => String(x).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

/** 计算压缩率百分比：compressed/size（存档级）。无有效数据返回 null。 */
export function ratioPercent(size: number, compressed: number): number | null {
  if (size <= 0) return null;
  return Math.min(100, Math.round((compressed / size) * 100));
}
