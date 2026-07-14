import { PrayError } from "../errors.js";
import {
  activeInvocationContext,
  lockfilePathForContext,
  projectInvocationContextFromCurrentDirectory,
  projectInvocationContextFromOptions,
  setActiveInvocationContext,
  type ProjectInvocationContext,
  type ProjectInvocationOptions,
} from "../project-context/index.js";
import { defaultResolveOptions, type ResolveOptions } from "../resolve/context.js";
import {
  resolveProject,
  resolveProjectWithGitRefreshFallback,
} from "../resolve/project.js";
import type { ResolvedProject } from "../resolve/types.js";

export function initializeInvocation(argumentsList: string[]): string[] {
  const { options, remaining } = parseGlobalOptions(argumentsList);
  const context = projectInvocationContextFromOptions(options);
  setActiveInvocationContext(context);
  return remaining;
}

export function manifestPath(): string {
  return invocationContextValue().manifestPath;
}

export function projectRoot(): string {
  return invocationContextValue().projectRoot;
}

export function lockfilePath(): string {
  return lockfilePathForContext(invocationContextValue());
}

export function selectedEnvironment(): string | undefined {
  return invocationContextValue().environment;
}

export async function resolveCurrentProject(
  options: ResolveOptions = defaultResolveOptions(),
): Promise<ResolvedProject> {
  const context = invocationContextValue();
  const resolveOptions: ResolveOptions = {
    ...options,
    environment: options.environment ?? context.environment,
  };
  return resolveProject(context.manifestPath, resolveOptions);
}

export async function resolveCurrentProjectWithGitRefreshFallback(
  options: ResolveOptions = defaultResolveOptions(),
  allowGitRefreshFallback = false,
): Promise<ResolvedProject> {
  const context = invocationContextValue();
  const resolveOptions: ResolveOptions = {
    ...options,
    environment: options.environment ?? context.environment,
  };
  return resolveProjectWithGitRefreshFallback(
    context.manifestPath,
    resolveOptions,
    allowGitRefreshFallback,
  );
}

function invocationContextValue(): ProjectInvocationContext {
  return (
    activeInvocationContext() ?? projectInvocationContextFromCurrentDirectory()
  );
}

function parseGlobalOptions(argumentsList: string[]): {
  options: ProjectInvocationOptions;
  remaining: string[];
} {
  const options: ProjectInvocationOptions = {};
  const remaining: string[] = [];
  let index = 0;
  while (index < argumentsList.length) {
    const argument = argumentsList[index]!;
    switch (argument) {
      case "--path":
        options.projectRoot = requireOptionValue("--path", argumentsList[++index]);
        index++;
        continue;
      case "--file-path":
        options.manifestPath = requireOptionValue(
          "--file-path",
          argumentsList[++index],
        );
        index++;
        continue;
      case "--env":
      case "--environment":
        options.environment = requireEnvironmentValue(
          argument,
          argumentsList[++index],
        );
        index++;
        continue;
      default:
        if (isTopLevelCommand(argument)) {
          remaining.push(...argumentsList.slice(index));
          return { options, remaining };
        }
        if (argument.startsWith("-")) {
          remaining.push(...argumentsList.slice(index));
          return { options, remaining };
        }
        remaining.push(...argumentsList.slice(index));
        return { options, remaining };
    }
  }
  return { options, remaining };
}

function requireOptionValue(flag: string, value: string | undefined): string {
  if (value === undefined) {
    throw PrayError.usage(`${flag} requires a value`);
  }
  return value;
}

function requireEnvironmentValue(
  flag: string,
  value: string | undefined,
): string {
  if (value === undefined) {
    throw PrayError.usage(`${flag} requires a value`);
  }
  return value;
}

function isTopLevelCommand(token: string): boolean {
  return [
    "manifest",
    "init",
    "prayer",
    "repo",
    "install",
    "add",
    "remove",
    "update",
    "unlock",
    "render",
    "plan",
    "apply",
    "verify",
    "drift",
    "format",
    "package",
    "publish",
    "login",
    "serve",
    "confess",
    "list",
    "outdated",
    "explain",
    "vendor",
    "clean",
    "tree",
    "sync",
    "trust",
    "version",
    "-V",
    "--version",
    "-h",
    "--help",
  ].includes(token);
}
