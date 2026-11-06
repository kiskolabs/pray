#!/usr/bin/env node
// Relative TS import cycle check for package entrypoints (madge --circular replacement).
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, extname, join, normalize, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const IMPORT_RE =
  /(?:import|export)\s+(?:type\s+)?(?:[^"'`]*?\s+from\s+)?["'](\.[^"']+)["']/g;

function normalizePath(path) {
  return normalize(resolve(path));
}

export function collectTypeScriptFiles(rootDir) {
  const files = [];
  const stack = [normalizePath(rootDir)];
  while (stack.length > 0) {
    const current = stack.pop();
    for (const entry of readdirSync(current)) {
      if (entry === "node_modules" || entry === "dist" || entry === "dist-test") continue;
      const full = join(current, entry);
      const info = statSync(full);
      if (info.isDirectory()) {
        stack.push(full);
        continue;
      }
      if (extname(full) === ".ts") files.push(normalizePath(full));
    }
  }
  return files;
}

export function resolveImport(fromFile, specifier) {
  const base = resolve(dirname(fromFile), specifier);
  const candidates = [base, `${base}.ts`, `${base}.js`, join(base, "index.ts"), join(base, "index.js")];
  if (base.endsWith(".js")) candidates.unshift(`${base.slice(0, -3)}.ts`);
  for (const candidate of candidates) {
    if (existsSync(candidate) && statSync(candidate).isFile()) return normalizePath(candidate);
  }
  return null;
}

export function buildImportGraph(files) {
  const fileSet = new Set(files);
  const graph = new Map();
  for (const file of files) {
    const source = readFileSync(file, "utf8");
    const deps = new Set();
    for (const match of source.matchAll(IMPORT_RE)) {
      const resolved = resolveImport(file, match[1]);
      if (resolved && fileSet.has(resolved)) deps.add(resolved);
    }
    graph.set(file, [...deps]);
  }
  return graph;
}

export function findImportCycles(graph) {
  const cycles = [];
  const visiting = new Set();
  const visited = new Set();
  const stack = [];
  function visit(node) {
    if (visiting.has(node)) {
      cycles.push(stack.slice(stack.indexOf(node)).concat(node));
      return;
    }
    if (visited.has(node)) return;
    visiting.add(node);
    stack.push(node);
    for (const next of graph.get(node) || []) visit(next);
    stack.pop();
    visiting.delete(node);
    visited.add(node);
  }
  for (const node of graph.keys()) visit(node);
  return cycles;
}

export function checkImportCycles(rootDir, entryFiles) {
  const root = normalizePath(rootDir);
  const graph = buildImportGraph(collectTypeScriptFiles(root));
  const reachable = new Set();
  const queue = entryFiles.map((entry) => normalizePath(entry));
  while (queue.length > 0) {
    const current = queue.pop();
    if (reachable.has(current) || !graph.has(current)) continue;
    reachable.add(current);
    for (const next of graph.get(current) || []) queue.push(next);
  }
  const subgraph = new Map(
    [...reachable].map((file) => [
      file,
      (graph.get(file) || []).filter((dep) => reachable.has(dep)),
    ]),
  );
  return findImportCycles(subgraph).map((cycle) =>
    cycle.map((file) => relative(root, file).split("\\").join("/")),
  );
}

export function main(argv) {
  if (argv.length < 2) {
    console.error("usage: check-import-cycles.mjs <src-root> <entry.ts>...");
    process.exitCode = 2;
    return;
  }
  const [rootDir, ...entries] = argv;
  const cycles = checkImportCycles(rootDir, entries);
  if (cycles.length === 0) {
    console.log("No circular dependency found!");
    return;
  }
  console.error("Circular dependencies:");
  for (const cycle of cycles) console.error(`  ${cycle.join(" -> ")}`);
  process.exitCode = 1;
}

if (
  process.argv[1] &&
  normalizePath(process.argv[1]) === normalizePath(fileURLToPath(import.meta.url))
) {
  main(process.argv.slice(2));
}
