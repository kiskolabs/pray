import { validateArchiveMemberPath } from "../archive/path-safety.js";
import type { PackageSpec } from "./types.js";

export function validateRelativePaths(spec: PackageSpec): void {
  for (const file of spec.files) {
    validateArchiveMemberPath(file);
  }
  for (const exportEntry of spec.exports.values()) {
    validateArchiveMemberPath(exportEntry.path);
    if (exportEntry.defaultPath) {
      validateArchiveMemberPath(exportEntry.defaultPath);
    }
  }
  for (const skill of spec.skills.values()) {
    validateArchiveMemberPath(skill.path);
  }
  for (const template of spec.templates.values()) {
    validateArchiveMemberPath(template.path);
  }
}
