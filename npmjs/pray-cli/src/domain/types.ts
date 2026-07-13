export type SourceKind =
  | "path"
  | "git"
  | "registry"
  | "pray_ssh"
  | "static index";

export type RenderMode = "managed" | "verbatim";

export type RenderConflict = "fail" | "warn" | "merge";

export type RenderChurn = "minimal" | "full";

export type LineEndings = "lf" | "crlf" | "native";

export type LocalPosition = "before" | "after";

export type PackageExportKind = "fragment" | "folder" | "skill";
