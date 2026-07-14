import { realpathSync } from "node:fs";
import { isAbsolute, resolve } from "node:path";
import { loadDotenvVariables } from "./dotenv.js";

export { setActiveInvocationContext, activeInvocationContext } from "./runtime.js";

export const ENV_PROJECT_PATH = "PRAY_PATH";
export const ENV_MANIFEST_PATH = "PRAY_FILE_PATH";
export const ENV_ENVIRONMENT = "PRAY_ENV";

export interface ProjectInvocationContext {
  projectRoot: string;
  manifestPath: string;
  environment?: string;
}

export interface ProjectInvocationOptions {
  projectRoot?: string;
  manifestPath?: string;
  environment?: string;
}

export function projectInvocationContextFromOptions(
  options: ProjectInvocationOptions = {},
): ProjectInvocationContext {
  const workingDirectory = process.cwd();
  const dotenv = loadDotenvVariables(workingDirectory);
  const projectRootHint =
    options.projectRoot ??
    envValue(ENV_PROJECT_PATH) ??
    dotenv.get(ENV_PROJECT_PATH) ??
    workingDirectory;
  const projectRoot = canonicalizePath(workingDirectory, projectRootHint);
  const manifestHint =
    options.manifestPath ??
    envValue(ENV_MANIFEST_PATH) ??
    dotenv.get(ENV_MANIFEST_PATH) ??
    "Prayfile";
  const manifestPath = isAbsolute(manifestHint)
    ? manifestHint
    : resolve(projectRoot, manifestHint);
  const environment = normalizeEnvironment(
    options.environment ??
      envValue(ENV_ENVIRONMENT) ??
      dotenv.get(ENV_ENVIRONMENT),
  );
  return {
    projectRoot,
    manifestPath,
    ...(environment ? { environment } : {}),
  };
}

export function projectInvocationContextFromCurrentDirectory(): ProjectInvocationContext {
  return projectInvocationContextFromOptions();
}

export function lockfilePathForContext(context: ProjectInvocationContext): string {
  return resolve(context.projectRoot, "Prayfile.lock");
}

function envValue(key: string): string | undefined {
  const value = process.env[key];
  if (value === undefined) {
    return undefined;
  }
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : undefined;
}

function normalizeEnvironment(value: string | undefined): string | undefined {
  if (value === undefined) {
    return undefined;
  }
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : undefined;
}

function canonicalizePath(base: string, path: string): string {
  const resolved = isAbsolute(path) ? path : resolve(base, path);
  try {
    return realpathSync.native(resolved);
  } catch {
    return resolved;
  }
}
