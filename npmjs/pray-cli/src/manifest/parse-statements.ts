import { normalizeVersionConstraint } from "../constraint.js";
import { PrayError } from "../errors.js";
import type { SourceKind } from "../domain/types.js";
import {
  keywordArray,
  keywordValue,
  parseCall,
  requirePositionalString,
  stringFromLiteral,
  stringFromValue,
} from "../literal/call-parser.js";
import { parseLiteral } from "../literal/parser.js";
import {
  literalAsBool,
  literalAsInteger,
} from "../literal/types.js";
import type {
  ManifestLocal,
  ManifestPackage,
  ManifestSource,
  ManifestTarget,
  RenderPolicy,
} from "./types.js";

const PARSE_CONTEXT = "manifest";

export function parseSource(rest: string): ManifestSource {
  const { values, keywords } = parseCall(rest);
  if (values.length === 0) {
    throw PrayError.parse(PARSE_CONTEXT, "source requires a name");
  }
  if (values.length < 2 && !keywords.has("path") && !keywords.has("git")) {
    throw PrayError.parse(
      PARSE_CONTEXT,
      "source requires a name and url, path:, or git:",
    );
  }
  const name = requirePositionalString(values, 0, PARSE_CONTEXT);
  let kind: SourceKind;
  let url: string;
  if (keywords.has("path")) {
    kind = "path";
    url = stringFromValue(keywordValue(keywords, "path", PARSE_CONTEXT), PARSE_CONTEXT);
  } else if (keywords.has("git")) {
    url = stringFromValue(keywordValue(keywords, "git", PARSE_CONTEXT), PARSE_CONTEXT);
    if (!url.startsWith("git+")) {
      url = `git+${url}`;
    }
    kind = "git";
  } else {
    url = requirePositionalString(values, 1, PARSE_CONTEXT);
    if (url.startsWith("git+")) {
      kind = "git";
    } else if (url.startsWith("pray+ssh://") || url.startsWith("ssh+pray://")) {
      kind = "pray_ssh";
    } else {
      kind = "registry";
    }
  }
  const subdir =
    keywords.get("subdir") ?? keywords.get("distribution");
  return {
    name,
    kind,
    url,
    ...(subdir ? { subdir: stringFromValue(subdir, PARSE_CONTEXT) } : {}),
    ...(keywords.has("rev")
      ? { rev: stringFromValue(keywordValue(keywords, "rev", PARSE_CONTEXT), PARSE_CONTEXT) }
      : {}),
    ...(keywords.has("tag")
      ? { tag: stringFromValue(keywordValue(keywords, "tag", PARSE_CONTEXT), PARSE_CONTEXT) }
      : {}),
  };
}

export function parseTargetHeader(rest: string): {
  target: ManifestTarget;
  isBlock: boolean;
} {
  const isBlock = rest.trimEnd().endsWith("do");
  const header = isBlock ? rest.trimEnd().slice(0, -2).trim() : rest.trim();
  const { values, keywords } = parseCall(header);
  const name = requirePositionalString(values, 0, PARSE_CONTEXT);
  const skills = [
    ...keywordArray(keywords, "folder"),
    ...keywordArray(keywords, "skills"),
  ];
  return {
    target: {
      name,
      outputs: keywordArray(keywords, "output"),
      skills,
      commands: keywordArray(keywords, "commands"),
      rules: keywordArray(keywords, "rules"),
      ...(keywords.has("max_bytes")
        ? {
            maxBytes: literalAsInteger(
              keywordValue(keywords, "max_bytes", PARSE_CONTEXT),
            ),
          }
        : {}),
    },
    isBlock,
  };
}

export function parseGroupHeader(rest: string): { name: string; isBlock: boolean } {
  const isBlock = rest.trimEnd().endsWith("do");
  const header = isBlock ? rest.trimEnd().slice(0, -2).trim() : rest.trim();
  const { values } = parseCall(header);
  return { name: requirePositionalString(values, 0, PARSE_CONTEXT), isBlock };
}

export function parsePackageDecl(rest: string): ManifestPackage {
  const { values, keywords } = parseCall(rest);
  if (values.length === 0) {
    throw PrayError.parse(PARSE_CONTEXT, "agent missing name");
  }
  const name = requirePositionalString(values, 0, PARSE_CONTEXT);
  const constraint = values[1]
    ? normalizeVersionConstraint(
        stringFromValue(values[1], PARSE_CONTEXT),
      )
    : "*";
  return {
    name,
    constraint,
    source: keywords.has("source")
      ? stringFromValue(keywordValue(keywords, "source", PARSE_CONTEXT), PARSE_CONTEXT)
      : undefined,
    exports: keywordArray(keywords, "exports"),
    targets: keywordArray(keywords, "targets"),
    features: keywordArray(keywords, "features"),
    optional: keywords.has("optional")
      ? literalAsBool(keywordValue(keywords, "optional", PARSE_CONTEXT)) ?? false
      : false,
    path: keywords.has("path")
      ? stringFromValue(keywordValue(keywords, "path", PARSE_CONTEXT), PARSE_CONTEXT)
      : undefined,
    git: keywords.has("git")
      ? stringFromValue(keywordValue(keywords, "git", PARSE_CONTEXT), PARSE_CONTEXT)
      : undefined,
    tag: keywords.has("tag")
      ? stringFromValue(keywordValue(keywords, "tag", PARSE_CONTEXT), PARSE_CONTEXT)
      : undefined,
    rev: keywords.has("rev")
      ? stringFromValue(keywordValue(keywords, "rev", PARSE_CONTEXT), PARSE_CONTEXT)
      : undefined,
    tarball: keywords.has("tarball")
      ? stringFromValue(keywordValue(keywords, "tarball", PARSE_CONTEXT), PARSE_CONTEXT)
      : undefined,
    oci: keywords.has("oci")
      ? stringFromValue(keywordValue(keywords, "oci", PARSE_CONTEXT), PARSE_CONTEXT)
      : undefined,
  };
}

export function parseLocalDecl(rest: string): ManifestLocal {
  const { values, keywords } = parseCall(rest);
  return {
    path: requirePositionalString(values, 0, PARSE_CONTEXT),
    position: keywords.has("position")
      ? (stringFromValue(
          keywordValue(keywords, "position", PARSE_CONTEXT),
          PARSE_CONTEXT,
        ) as ManifestLocal["position"])
      : "after",
    optional: keywords.has("optional")
      ? literalAsBool(keywordValue(keywords, "optional", PARSE_CONTEXT)) ?? false
      : false,
  };
}

export function parseRenderPolicy(rest: string): RenderPolicy {
  const { keywords } = parseCall(rest);
  return {
    mode: keywords.has("mode")
      ? (stringFromValue(
          keywordValue(keywords, "mode", PARSE_CONTEXT),
          PARSE_CONTEXT,
        ) as RenderPolicy["mode"])
      : "managed",
    conflict: keywords.has("conflict")
      ? (stringFromValue(
          keywordValue(keywords, "conflict", PARSE_CONTEXT),
          PARSE_CONTEXT,
        ) as RenderPolicy["conflict"])
      : "fail",
    churn: keywords.has("churn")
      ? (stringFromValue(
          keywordValue(keywords, "churn", PARSE_CONTEXT),
          PARSE_CONTEXT,
        ) as RenderPolicy["churn"])
      : "minimal",
    header: keywords.has("header")
      ? literalAsBool(keywordValue(keywords, "header", PARSE_CONTEXT)) ?? true
      : true,
    sectionMarkers: keywords.has("section_markers")
      ? literalAsBool(keywordValue(keywords, "section_markers", PARSE_CONTEXT)) ?? true
      : true,
    lineEndings: keywords.has("line_endings")
      ? (stringFromValue(
          keywordValue(keywords, "line_endings", PARSE_CONTEXT),
          PARSE_CONTEXT,
        ) as RenderPolicy["lineEndings"])
      : "lf",
  };
}

export function applyTargetStatement(target: ManifestTarget, statement: string): void {
  if (statement.startsWith("output ")) {
    target.outputs.push(
      stringFromLiteral(statement.slice("output ".length), PARSE_CONTEXT),
    );
    return;
  }
  if (statement.startsWith("folder ") || statement.startsWith("skills ")) {
    const rest = statement.startsWith("folder ")
      ? statement.slice("folder ".length)
      : statement.slice("skills ".length);
    target.skills.push(stringFromLiteral(rest, PARSE_CONTEXT));
    return;
  }
  if (statement.startsWith("commands ")) {
    target.commands.push(
      stringFromLiteral(statement.slice("commands ".length), PARSE_CONTEXT),
    );
    return;
  }
  if (statement.startsWith("rules ")) {
    target.rules.push(
      stringFromLiteral(statement.slice("rules ".length), PARSE_CONTEXT),
    );
    return;
  }
  if (statement.startsWith("max_bytes ")) {
    const value = parseLiteral(statement.slice("max_bytes ".length).trim());
    target.maxBytes = literalAsInteger(value);
    return;
  }
  throw PrayError.parse(PARSE_CONTEXT, `unrecognized target statement: ${statement}`);
}
