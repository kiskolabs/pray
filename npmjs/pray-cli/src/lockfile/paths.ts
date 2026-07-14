import { relative, resolve } from "node:path";
import { activeInvocationContext } from "../project-context/runtime.js";

export function relativeLockfilePath(projectRoot: string, targetPath: string): string {
  const absolute = resolve(projectRoot, targetPath);
  const normalizedRoot = lexicalNormalizePath(resolve(projectRoot));
  const normalizedAbsolute = lexicalNormalizePath(absolute);
  let relativePath = normalizedAbsolute.startsWith(normalizedRoot)
    ? normalizedAbsolute.slice(normalizedRoot.length).replace(/^[/\\]/, "")
    : targetPath;
  relativePath = relativePath.replace(/\\/g, "/");
  if (relativePath === "." || relativePath.startsWith("./")) {
    return relativePath;
  }
  return `./${relativePath}`;
}

export function normalizeLockfileArtifact(
  projectRoot: string,
  artifact: string,
  packageRoot: string,
): string {
  if (!artifact.startsWith("path:")) {
    return artifact;
  }
  const pathText = artifact.slice("path:".length);
  const relative = pathText.startsWith("/")
    ? relativeLockfilePath(projectRoot, pathText)
    : relativeLockfilePath(projectRoot, packageRoot);
  return `path:${relative}`;
}

function lexicalNormalizePath(path: string): string {
  const segments = path.split(/[/\\]/);
  const normalized: string[] = [];
  for (const segment of segments) {
    if (segment.length === 0 || segment === ".") {
      continue;
    }
    if (segment === "..") {
      normalized.pop();
      continue;
    }
    normalized.push(segment);
  }
  const prefix = path.startsWith("/") ? "/" : "";
  return `${prefix}${normalized.join("/")}`;
}

export function projectRootFromManifest(manifestPath: string): string {
  const parent = resolve(manifestPath, "..");
  return parent.length === 0 ? "." : parent;
}

export function defaultManifestPath(workingDirectory = process.cwd()): string {
  const active = activeInvocationContext();
  if (active) {
    return active.manifestPath;
  }
  return resolve(workingDirectory, "Prayfile");
}

export function defaultProjectRoot(workingDirectory = process.cwd()): string {
  const active = activeInvocationContext();
  if (active) {
    return active.projectRoot;
  }
  return projectRootFromManifest(defaultManifestPath(workingDirectory));
}

export function defaultLockfilePath(projectRoot: string): string {
  return resolve(projectRoot, "Prayfile.lock");
}

export function relativeProjectPath(projectRoot: string, absolutePath: string): string {
  return relative(projectRoot, absolutePath).replace(/\\/g, "/");
}
