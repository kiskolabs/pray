export type SourceKind =
  | "path"
  | "git"
  | "registry"
  | "pray_ssh"
  | "static index";

export type RenderMode = "managed" | "verbatim";

export type RenderConflict = "fail";

export type RenderChurn = "minimal" | "full";

export type LocalPosition = "before" | "after";

export type PackageExportKind = "fragment" | "folder" | "skill" | "file";
