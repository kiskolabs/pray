import { PrayError } from "../errors.js";
import { parseCall, stringFromValue } from "../literal/call-parser.js";
import type { StatementReader } from "../literal/statements.js";
import { parsePackageDecl } from "./parse-statements.js";
import type { Manifest, ManifestPublishRemote } from "./types.js";

const PARSE_CONTEXT = "manifest";

export function applyPublishOrUnrecognized(
  reader: StatementReader,
  manifest: Manifest,
  statement: string,
): void {
  if (!statement.startsWith("publish ")) {
    throw PrayError.parse(
      PARSE_CONTEXT,
      `unrecognized statement: ${statement}`,
    );
  }
  const remotes = manifest.publishRemotes ?? [];
  remotes.push(
    parsePublishStatement(reader, statement.slice("publish ".length)),
  );
  manifest.publishRemotes = remotes;
}

function parsePublishStatement(
  reader: StatementReader,
  rest: string,
): ManifestPublishRemote {
  const isBlock = rest.trimEnd().endsWith(" do");
  const header = rest.replace(/\s+do\s*$/, "").trim();
  const remote = parsePublishHeader(header);
  if (isBlock) {
    parsePublishBlock(reader, remote);
  }
  return remote;
}

function parsePublishHeader(rest: string): ManifestPublishRemote {
  const { values, keywords } = parseCall(rest);
  if (values.length === 0) {
    throw PrayError.parse(PARSE_CONTEXT, "publish requires a name");
  }
  const name = stringFromValue(values[0]!, PARSE_CONTEXT);
  if (
    keywords.has("git") ||
    keywords.has("source") ||
    keywords.has("signing_key") ||
    keywords.has("token")
  ) {
    throw PrayError.parse(
      PARSE_CONTEXT,
      `publish "${name}" does not take git:, source:, signing_key:, or token:`,
    );
  }
  return {
    name,
    path: keywords.has("path")
      ? stringFromValue(keywords.get("path")!, PARSE_CONTEXT)
      : undefined,
    url: values[1] ? stringFromValue(values[1], PARSE_CONTEXT) : undefined,
    packages: [],
  };
}

function parsePublishBlock(
  reader: StatementReader,
  remote: ManifestPublishRemote,
): void {
  while (true) {
    const statement = reader.nextStatement();
    if (statement === undefined) {
      throw PrayError.parse(PARSE_CONTEXT, "missing 'end' for publish block");
    }
    if (statement === "end") {
      return;
    }
    const prayRest = statement.startsWith("pray ")
      ? statement.slice("pray ".length)
      : statement.startsWith("use ")
        ? statement.slice("use ".length)
        : statement.startsWith("include ")
          ? statement.slice("include ".length)
          : statement.startsWith("agent ")
            ? statement.slice("agent ".length)
            : statement.startsWith("package ")
              ? statement.slice("package ".length)
              : undefined;
    if (prayRest === undefined) {
      throw PrayError.parse(
        PARSE_CONTEXT,
        `publish blocks only support pray package names: ${statement}`,
      );
    }
    const packageEntry = parsePackageDecl(prayRest);
    if (remote.packages.includes(packageEntry.name)) {
      throw PrayError.parse(
        PARSE_CONTEXT,
        `duplicate package ${packageEntry.name} in publish "${remote.name}"`,
      );
    }
    remote.packages.push(packageEntry.name);
  }
}
