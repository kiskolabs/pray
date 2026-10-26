/** True when a path is absolute on POSIX or Windows (drive letter or UNC). */
export function isAbsoluteFilesystemPath(path: string): boolean {
  if (path.startsWith("/") || path.startsWith("\\")) {
    return true;
  }
  if (/^[A-Za-z]:[/\\]/.test(path)) {
    return true;
  }
  return path.startsWith("\\\\");
}
