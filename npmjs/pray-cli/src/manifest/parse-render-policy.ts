import { PrayError } from "../errors.js";
import {
  keywordValue,
  parseCall,
  stringFromValue,
} from "../literal/call-parser.js";
import { literalAsBool } from "../literal/types.js";
import type { RenderPolicy } from "./types.js";

const PARSE_CONTEXT = "manifest";
const RENDER_KEYS = new Set(["mode", "conflict", "churn", "header"]);

export function parseRenderPolicy(rest: string): RenderPolicy {
  const { keywords } = parseCall(rest);
  for (const key of keywords.keys()) {
    if (!RENDER_KEYS.has(key)) {
      throw PrayError.parse(PARSE_CONTEXT, `render does not accept ${key}`);
    }
  }
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
  const mode = keywords.has("mode")
    ? stringFromValue(
        keywordValue(keywords, "mode", PARSE_CONTEXT),
        PARSE_CONTEXT,
      )
    : "managed";
  if (mode !== "managed") {
    throw PrayError.unsupported(`render mode :${mode} is not implemented`);
  }
  const churn = keywords.has("churn")
    ? stringFromValue(
        keywordValue(keywords, "churn", PARSE_CONTEXT),
        PARSE_CONTEXT,
      )
    : "minimal";
  if (churn !== "minimal") {
    throw PrayError.unsupported(`render churn :${churn} is not implemented`);
  }
  return {
    mode,
    conflict,
    churn,
    header: keywords.has("header")
      ? (literalAsBool(keywordValue(keywords, "header", PARSE_CONTEXT)) ?? true)
      : true,
  };
}
