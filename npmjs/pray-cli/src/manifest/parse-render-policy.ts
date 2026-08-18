import { PrayError } from "../errors.js";
import {
  keywordValue,
  parseCall,
  stringFromValue,
} from "../literal/call-parser.js";
import { literalAsBool } from "../literal/types.js";
import type { RenderPolicy } from "./types.js";

const PARSE_CONTEXT = "manifest";

export function parseRenderPolicy(rest: string): RenderPolicy {
  const { keywords } = parseCall(rest);
  const conflict = keywords.has("conflict")
    ? stringFromValue(
        keywordValue(keywords, "conflict", PARSE_CONTEXT),
        PARSE_CONTEXT,
      )
    : "fail";
  if (conflict !== "fail") {
    throw PrayError.unsupported(
      `render conflict :${conflict} is not implemented; only :fail is supported`,
    );
  }
  return {
    mode: keywords.has("mode")
      ? (stringFromValue(
          keywordValue(keywords, "mode", PARSE_CONTEXT),
          PARSE_CONTEXT,
        ) as RenderPolicy["mode"])
      : "managed",
    conflict,
    churn: keywords.has("churn")
      ? (stringFromValue(
          keywordValue(keywords, "churn", PARSE_CONTEXT),
          PARSE_CONTEXT,
        ) as RenderPolicy["churn"])
      : "minimal",
    header: keywords.has("header")
      ? (literalAsBool(keywordValue(keywords, "header", PARSE_CONTEXT)) ?? true)
      : true,
    sectionMarkers: keywords.has("section_markers")
      ? (literalAsBool(
          keywordValue(keywords, "section_markers", PARSE_CONTEXT),
        ) ?? true)
      : true,
    lineEndings: keywords.has("line_endings")
      ? (stringFromValue(
          keywordValue(keywords, "line_endings", PARSE_CONTEXT),
          PARSE_CONTEXT,
        ) as RenderPolicy["lineEndings"])
      : "lf",
  };
}
