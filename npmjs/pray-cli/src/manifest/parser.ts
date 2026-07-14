import { PrayError } from "../errors.js";
import { stringFromLiteral } from "../literal/call-parser.js";
import { prepareParserLines } from "../literal/lines.js";
import { StatementReader } from "../literal/statements.js";
import {
  applyTargetStatement,
  parseGroupHeader,
  parseLocalDecl,
  parsePackageDecl,
  parseRenderPolicy,
  parseSource,
  parseTargetHeader,
} from "./parse-statements.js";
import { defaultRenderPolicy, type Manifest } from "./types.js";

export function parseManifestText(text: string): Manifest {
  const parser = new BlockParser(prepareParserLines(text));
  return parser.parse();
}

class BlockParser {
  private readonly reader: StatementReader;
  private readonly groupStack: string[][] = [];

  constructor(lines: string[]) {
    this.reader = new StatementReader(lines);
  }

  parse(): Manifest {
    const manifest: Manifest = {
      prayfileVersion: "",
      sources: [],
      targets: [],
      packages: [],
      local: [],
      render: defaultRenderPolicy(),
    };

    while (true) {
      const statement = this.reader.nextStatement();
      if (statement === undefined) {
        break;
      }
      if (statement === "end") {
        throw PrayError.parse("manifest", "unexpected 'end'");
      }
      this.applyStatement(manifest, statement, false);
    }

    if (manifest.prayfileVersion.length === 0) {
      throw PrayError.manifest("missing prayfile version");
    }

    return manifest;
  }

  private applyStatement(
    manifest: Manifest,
    statement: string,
    allowTarget: boolean,
  ): void {
    if (statement.startsWith("prayfile ")) {
      manifest.prayfileVersion = stringFromLiteral(
        statement.slice("prayfile ".length),
        "manifest",
      );
      return;
    }
    if (statement.startsWith("source ")) {
      manifest.sources.push(parseSource(statement.slice("source ".length)));
      return;
    }
    if (statement.startsWith("target ")) {
      if (!allowTarget && !statement.endsWith(" do")) {
        throw PrayError.parse("manifest", "target must use a block");
      }
      const { target, isBlock } = parseTargetHeader(statement.slice("target ".length));
      manifest.targets.push(target);
      if (isBlock) {
        const index = manifest.targets.length - 1;
        this.parseTargetBlock(manifest, index);
      }
      return;
    }
    if (statement.startsWith("group ")) {
      const { groups, isBlock } = parseGroupHeader(statement.slice("group ".length));
      if (!isBlock) {
        throw PrayError.parse("manifest", "group must use a block");
      }
      if (this.groupStack.length > 0) {
        throw PrayError.parse("manifest", "nested group blocks are not supported");
      }
      this.groupStack.push(groups);
      this.parseGroupBlock(manifest);
      this.groupStack.pop();
      return;
    }
    if (statement.startsWith("agent ")) {
      manifest.packages.push(
        this.parsePackageWithGroups(statement.slice("agent ".length)),
      );
      return;
    }
    if (statement.startsWith("local ")) {
      manifest.local.push(parseLocalDecl(statement.slice("local ".length)));
      return;
    }
    if (statement.startsWith("render ")) {
      manifest.render = parseRenderPolicy(statement.slice("render ".length));
      return;
    }
    throw PrayError.parse("manifest", `unrecognized statement: ${statement}`);
  }

  private parseGroupBlock(manifest: Manifest): void {
    while (true) {
      const statement = this.reader.nextStatement();
      if (statement === undefined) {
        throw PrayError.parse("manifest", "missing 'end' for group block");
      }
      if (statement === "end") {
        return;
      }
      if (statement.startsWith("group ")) {
        throw PrayError.parse("manifest", "nested group blocks are not supported");
      }
      if (statement.startsWith("agent ")) {
        manifest.packages.push(
          this.parsePackageWithGroups(statement.slice("agent ".length)),
        );
        continue;
      }
      if (statement.startsWith("package ")) {
        manifest.packages.push(
          this.parsePackageWithGroups(statement.slice("package ".length)),
        );
        continue;
      }
      throw PrayError.parse(
        "manifest",
        `group blocks only support agent or package declarations: ${statement}`,
      );
    }
  }

  private parsePackageWithGroups(rest: string) {
    const packageEntry = parsePackageDecl(rest);
    packageEntry.groups = [...(this.groupStack[this.groupStack.length - 1] ?? [])];
    return packageEntry;
  }

  private parseTargetBlock(manifest: Manifest, targetIndex: number): void {
    while (true) {
      const statement = this.reader.nextStatement();
      if (statement === undefined) {
        throw PrayError.parse("manifest", "missing 'end' for target block");
      }
      if (statement === "end") {
        return;
      }
      const target = manifest.targets[targetIndex];
      if (!target) {
        throw PrayError.manifest("target index out of range");
      }
      applyTargetStatement(target, statement);
    }
  }
}
