import { dirname, resolve } from "node:path";
import { PrayError } from "pray-core/errors.js";
import { parseManifest, readManifestText } from "pray-core/manifest/index.js";
import type { Manifest, ManifestSource } from "pray-core/manifest/types.js";
import {
  type RegistrySearchHit,
  searchHttpRegistry,
  searchLocalRegistry,
} from "pray-core/registry/search.js";
import { manifestPath } from "../invocation.js";

interface SearchArguments {
  query: string;
  source?: string;
  root?: string;
  url?: string;
}

export async function runSearch(argumentsList: string[]): Promise<void> {
  const options = parseSearchArguments(argumentsList);
  const hits = options.root
    ? await searchLocalRegistry(options.root, options.query)
    : options.url
      ? await searchHttpRegistry(options.url, options.query)
      : await searchManifestSource(options.query, options.source);
  printHits(hits);
}

function parseSearchArguments(argumentsList: string[]): SearchArguments {
  let query: string | undefined;
  const options: Omit<SearchArguments, "query"> = {};
  for (let index = 0; index < argumentsList.length; index += 1) {
    const argument = argumentsList[index]!;
    if (
      argument === "--source" ||
      argument === "--root" ||
      argument === "--url"
    ) {
      const value = argumentsList[++index];
      if (value === undefined || value.startsWith("--")) {
        throw PrayError.usage(`${argument} requires a value`);
      }
      options[argument.slice(2) as "source" | "root" | "url"] = value;
    } else if (argument.startsWith("-")) {
      throw PrayError.usage(`unknown search flag: ${argument}`);
    } else if (query === undefined) {
      query = argument;
    } else {
      throw PrayError.usage(`unexpected search argument: ${argument}`);
    }
  }
  if (query === undefined) throw PrayError.usage("search requires a query");
  return { query, ...options };
}

async function searchManifestSource(
  query: string,
  sourceName: string | undefined,
): Promise<RegistrySearchHit[]> {
  let manifest: Manifest;
  try {
    manifest = parseManifest(readManifestText(manifestPath()));
  } catch (error) {
    if (error instanceof PrayError && error.kind === "io") {
      throw PrayError.unsupported(
        "search requires a Prayfile, or pass --root PATH / --url URL",
      );
    }
    throw error;
  }
  const source = selectSource(manifest.sources, sourceName);
  if (source.kind === "path") {
    return searchLocalRegistry(
      resolve(dirname(manifestPath()), source.url),
      query,
    );
  }
  if (source.kind === "registry" || source.kind === "static index") {
    return searchHttpRegistry(source.url, query);
  }
  throw PrayError.unsupported(
    `search does not support source kind ${source.kind}`,
  );
}

function selectSource(
  sources: ManifestSource[],
  sourceName: string | undefined,
): ManifestSource {
  if (sourceName !== undefined) {
    const source = sources.find((candidate) => candidate.name === sourceName);
    if (source === undefined) {
      throw PrayError.resolution(`unknown source: ${sourceName}`);
    }
    return source;
  }
  const source =
    sources.find(
      (candidate) =>
        candidate.kind === "registry" || candidate.kind === "static index",
    ) ?? sources[0];
  if (source === undefined) {
    throw PrayError.resolution(
      "Prayfile has no sources; pass --root PATH or --url URL",
    );
  }
  return source;
}

function printHits(hits: RegistrySearchHit[]): void {
  if (hits.length === 0) {
    process.stdout.write("no packages matched\n");
    return;
  }
  for (const hit of hits) {
    process.stdout.write(
      hit.summary === undefined
        ? `${hit.name}\n`
        : `${hit.name} — ${hit.summary}\n`,
    );
  }
}
