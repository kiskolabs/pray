import { PrayError } from "../errors.js";
import {
  parseCall,
  stringFromLiteral,
  stringFromValue,
} from "../literal/call-parser.js";
import { prepareParserLines } from "../literal/lines.js";
import { splitSymbolAssignment } from "../literal/statement-surface.js";
import { StatementReader } from "../literal/statements.js";
import { isPraySymbolKey } from "../substitute.js";
import {
  DEPRECATED_AGENT,
  DEPRECATED_OUTPUT,
  DEPRECATED_SKILLS,
  DEPRECATED_TARGET,
  noteDeprecatedKeyword,
} from "./deprecation.js";
import {
  bindLocalEntry,
  bindPackageEntry,
  destinationHeaderKeyword,
  isLocalPathForm,
  newDestinationTarget,
  packageRoles,
  roleForDestination,
  targetMode,
  upsertLocal,
  upsertPackage,
} from "./destination.js";
import {
  applyTargetStatement,
  parseGroupHeader,
  parseLocalDecl,
  parsePackageDecl,
  parseRenderPolicy,
  parseSource,
  parseTargetHeader,
} from "./parse-statements.js";
import {
  type DestinationMode,
  defaultRenderPolicy,
  type Manifest,
  type ManifestLocal,
  type ManifestPackage,
} from "./types.js";

const PARSE_CONTEXT = "manifest";

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
      symbols: {},
      render: defaultRenderPolicy(),
    };

    while (true) {
      const statement = this.reader.nextStatement();
      if (statement === undefined) {
        break;
      }
      if (statement === "end") {
        throw PrayError.parse(PARSE_CONTEXT, "unexpected 'end'");
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
        PARSE_CONTEXT,
      );
      return;
    }
    if (statement.startsWith("source ")) {
      manifest.sources.push(parseSource(statement.slice("source ".length)));
      return;
    }
    if (statement.startsWith("target ")) {
      if (!allowTarget && !statement.endsWith(" do")) {
        throw PrayError.parse(PARSE_CONTEXT, "target must use a block");
      }
      manifest.deprecatedKeywords = noteDeprecatedKeyword(
        manifest.deprecatedKeywords,
        DEPRECATED_TARGET,
      );
      const { target, isBlock } = parseTargetHeader(
        statement.slice("target ".length),
      );
      manifest.targets.push(target);
      if (isBlock) {
        const index = manifest.targets.length - 1;
        this.parseTargetBlock(manifest, index);
      }
      return;
    }
    if (statement.startsWith("group ")) {
      const { groups, isBlock } = parseGroupHeader(
        statement.slice("group ".length),
      );
      if (!isBlock) {
        throw PrayError.parse(PARSE_CONTEXT, "group must use a block");
      }
      if (this.groupStack.length > 0) {
        throw PrayError.parse(
          PARSE_CONTEXT,
          "nested group blocks are not supported",
        );
      }
      this.groupStack.push(groups);
      this.parseGroupBlock(manifest);
      this.groupStack.pop();
      return;
    }
    if (statement.startsWith("agent ")) {
      manifest.deprecatedKeywords = noteDeprecatedKeyword(
        manifest.deprecatedKeywords,
        DEPRECATED_AGENT,
      );
      upsertPackage(
        manifest,
        this.parsePackageWithGroups(statement.slice("agent ".length)),
      );
      return;
    }
    if (statement.startsWith("package ")) {
      upsertPackage(
        manifest,
        this.parsePackageWithGroups(statement.slice("package ".length)),
      );
      return;
    }
    if (statement === "pray do" || statement === "template do") {
      this.parseSymbolsBlock(manifest);
      return;
    }
    for (const prefix of ["pray ", "use ", "include "]) {
      if (statement.startsWith(prefix)) {
        this.applyPrayStatement(
          manifest,
          statement.slice(prefix.length),
          undefined,
        );
        return;
      }
    }
    if (statement.startsWith("compose ") || statement.startsWith("output ")) {
      if (statement.startsWith("output ")) {
        if (!statement.endsWith(" do")) {
          throw PrayError.parse(
            PARSE_CONTEXT,
            'top-level output must use a compose block (output "path" do ... end)',
          );
        }
        manifest.deprecatedKeywords = noteDeprecatedKeyword(
          manifest.deprecatedKeywords,
          DEPRECATED_OUTPUT,
        );
      }
      const rest = statement.startsWith("compose ")
        ? statement.slice("compose ".length)
        : statement.slice("output ".length);
      this.parseDestinationBlock(manifest, rest, "compose");
      return;
    }
    if (
      statement.startsWith("tree ") ||
      statement.startsWith("folder ") ||
      statement.startsWith("skills ")
    ) {
      const isFolderOrSkills =
        statement.startsWith("folder ") || statement.startsWith("skills ");
      if (isFolderOrSkills && !statement.endsWith(" do")) {
        throw PrayError.parse(
          PARSE_CONTEXT,
          "top-level folder/skills must use a tree block",
        );
      }
      const rest = statement.startsWith("tree ")
        ? statement.slice("tree ".length)
        : statement.startsWith("folder ")
          ? statement.slice("folder ".length)
          : statement.slice("skills ".length);
      if (statement.startsWith("skills ")) {
        manifest.deprecatedKeywords = noteDeprecatedKeyword(
          manifest.deprecatedKeywords,
          DEPRECATED_SKILLS,
        );
      }
      this.parseDestinationBlock(manifest, rest, "tree");
      return;
    }
    if (statement.startsWith("file ")) {
      this.parseFileBlock(manifest, statement.slice("file ".length));
      return;
    }
    if (statement.startsWith("local ")) {
      const local = parseLocalDecl(statement.slice("local ".length));
      local.bound = false;
      upsertLocal(manifest, local);
      return;
    }
    if (statement.startsWith("render ")) {
      manifest.render = parseRenderPolicy(statement.slice("render ".length));
      return;
    }
    throw PrayError.parse(
      PARSE_CONTEXT,
      `unrecognized statement: ${statement}`,
    );
  }

  private parseDestinationBlock(
    manifest: Manifest,
    rest: string,
    mode: DestinationMode,
  ): void {
    if (!rest.trimEnd().endsWith("do")) {
      throw PrayError.parse(
        PARSE_CONTEXT,
        `${mode === "compose" ? "compose" : "tree"} must use a block`,
      );
    }
    const header = rest.trimEnd().slice(0, -2).trim();
    const { values, keywords } = parseCall(header);
    const path = values[0]
      ? stringFromValue(values[0], PARSE_CONTEXT)
      : undefined;
    if (path === undefined) {
      throw PrayError.parse(PARSE_CONTEXT, "destination missing path");
    }
    const target = newDestinationTarget(mode, path);
    target.header = destinationHeaderKeyword(mode, keywords);
    manifest.targets.push(target);
    const index = manifest.targets.length - 1;
    while (true) {
      const statement = this.reader.nextStatement();
      if (statement === undefined) {
        throw PrayError.parse(
          PARSE_CONTEXT,
          "missing 'end' for destination block",
        );
      }
      if (statement === "end") {
        return;
      }
      const prefixes = ["pray ", "use ", "include ", "agent ", "package "];
      const prayPrefix = prefixes.find((prefix) =>
        statement.startsWith(prefix),
      );
      if (prayPrefix) {
        this.applyPrayStatement(
          manifest,
          statement.slice(prayPrefix.length),
          index,
        );
        continue;
      }
      if (mode === "compose" && statement.startsWith("local ")) {
        const local = parseLocalDecl(statement.slice("local ".length));
        local.bound = true;
        bindLocalEntry(manifest.targets[index]!, local.path);
        upsertLocal(manifest, local);
        continue;
      }
      throw PrayError.parse(
        PARSE_CONTEXT,
        `unsupported statement inside destination block: ${statement}`,
      );
    }
  }

  private parseSymbolsBlock(manifest: Manifest): void {
    while (true) {
      const statement = this.reader.nextStatement();
      if (statement === undefined) {
        throw PrayError.parse(
          PARSE_CONTEXT,
          "missing 'end' for pray/template block",
        );
      }
      if (statement === "end") {
        return;
      }
      const assignment = splitSymbolAssignment(statement);
      if (!assignment) {
        throw PrayError.parse(
          PARSE_CONTEXT,
          `unsupported statement inside pray/template block: ${statement}`,
        );
      }
      const { key, value: valueLiteral } = assignment;
      if (!isPraySymbolKey(key)) {
        throw PrayError.parse(
          PARSE_CONTEXT,
          `invalid pray symbol key \`${key}\``,
        );
      }
      if (Object.hasOwn(manifest.symbols, key)) {
        throw PrayError.parse(
          PARSE_CONTEXT,
          `duplicate pray symbol \`${key}\``,
        );
      }
      manifest.symbols[key] = stringFromLiteral(valueLiteral, PARSE_CONTEXT);
    }
  }

  private parseFileBlock(manifest: Manifest, rest: string): void {
    const isBlock = rest.trimEnd().endsWith("do");
    if (!isBlock) {
      throw PrayError.parse(
        PARSE_CONTEXT,
        'file must use a block (or use pray ..., file: "path")',
      );
    }
    const header = rest.trimEnd().slice(0, -2).trim();
    const { values } = parseCall(header);
    const filePath = values[0]
      ? stringFromValue(values[0], PARSE_CONTEXT)
      : undefined;
    if (filePath === undefined) {
      throw PrayError.parse(PARSE_CONTEXT, "file block missing path");
    }
    let sawPackage = false;
    while (true) {
      const statement = this.reader.nextStatement();
      if (statement === undefined) {
        throw PrayError.parse(PARSE_CONTEXT, "missing 'end' for file block");
      }
      if (statement === "end") {
        if (!sawPackage) {
          throw PrayError.parse(
            PARSE_CONTEXT,
            "file block requires a pray package declaration",
          );
        }
        return;
      }
      const prefixes = ["pray ", "use ", "include ", "agent ", "package "];
      const prayPrefix = prefixes.find((prefix) =>
        statement.startsWith(prefix),
      );
      if (prayPrefix) {
        const packageEntry = this.parsePackageWithGroups(
          statement.slice(prayPrefix.length),
        );
        if (packageEntry.file) {
          throw PrayError.parse(
            PARSE_CONTEXT,
            "file: keyword is invalid inside a file block",
          );
        }
        packageEntry.file = filePath;
        packageEntry.bound = true;
        const roles = packageRoles(packageEntry);
        if (!roles.includes("file")) {
          roles.push("file");
        }
        packageEntry.roles = roles;
        upsertPackage(manifest, packageEntry);
        sawPackage = true;
        continue;
      }
      throw PrayError.parse(
        PARSE_CONTEXT,
        `unsupported statement inside file block: ${statement}`,
      );
    }
  }

  private applyPrayStatement(
    manifest: Manifest,
    rest: string,
    destinationIndex: number | undefined,
  ): void {
    const { values, keywords } = parseCall(rest);
    if (values.length === 0) {
      throw PrayError.parse(PARSE_CONTEXT, "pray missing package or path");
    }
    const first = stringFromValue(values[0]!, PARSE_CONTEXT);
    const hasPackageSignal =
      values.length > 1 ||
      [
        "source",
        "export",
        "exports",
        "file",
        "optional",
        "path",
        "git",
        "tag",
        "rev",
        "tarball",
        "oci",
        "targets",
        "features",
      ].some((key) => keywords.has(key));

    const inCompose =
      destinationIndex !== undefined &&
      targetMode(manifest.targets[destinationIndex]!) === "compose";

    if (!hasPackageSignal && isLocalPathForm(first)) {
      if (!inCompose) {
        throw PrayError.parse(
          PARSE_CONTEXT,
          "local pray paths are only valid inside compose blocks",
        );
      }
      const local: ManifestLocal = {
        path: first,
        position: "after",
        optional: false,
        bound: true,
      };
      if (destinationIndex !== undefined) {
        bindLocalEntry(manifest.targets[destinationIndex]!, local.path);
      }
      upsertLocal(manifest, local);
      return;
    }

    const packageEntry: ManifestPackage = this.parsePackageWithGroups(rest);
    if (packageEntry.file) {
      if (destinationIndex !== undefined) {
        throw PrayError.parse(
          PARSE_CONTEXT,
          "file: is mutually exclusive with compose/tree nesting",
        );
      }
      packageEntry.bound = true;
      const roles = packageRoles(packageEntry);
      if (!roles.includes("file")) {
        roles.push("file");
      }
      packageEntry.roles = roles;
    }
    if (destinationIndex !== undefined) {
      const target = manifest.targets[destinationIndex]!;
      const mode = targetMode(target);
      packageEntry.bound = true;
      const role = roleForDestination(mode);
      if (role) {
        const roles = packageRoles(packageEntry);
        if (!roles.includes(role)) {
          roles.push(role);
        }
        packageEntry.roles = roles;
      }
      bindPackageEntry(target, packageEntry.name);
    }
    upsertPackage(manifest, packageEntry);
  }

  private parseGroupBlock(manifest: Manifest): void {
    while (true) {
      const statement = this.reader.nextStatement();
      if (statement === undefined) {
        throw PrayError.parse(PARSE_CONTEXT, "missing 'end' for group block");
      }
      if (statement === "end") {
        return;
      }
      if (statement.startsWith("agent ")) {
        manifest.deprecatedKeywords = noteDeprecatedKeyword(
          manifest.deprecatedKeywords,
          DEPRECATED_AGENT,
        );
        upsertPackage(
          manifest,
          this.parsePackageWithGroups(statement.slice("agent ".length)),
        );
        continue;
      }
      const prefix = ["package ", "pray ", "use "].find((candidate) =>
        statement.startsWith(candidate),
      );
      if (prefix) {
        upsertPackage(
          manifest,
          this.parsePackageWithGroups(statement.slice(prefix.length)),
        );
        continue;
      }
      throw PrayError.parse(
        PARSE_CONTEXT,
        `group blocks only support agent, package, or pray declarations: ${statement}`,
      );
    }
  }

  private parsePackageWithGroups(rest: string): ManifestPackage {
    const packageEntry = parsePackageDecl(rest);
    packageEntry.groups = [
      ...(this.groupStack[this.groupStack.length - 1] ?? []),
    ];
    return packageEntry;
  }

  private parseTargetBlock(manifest: Manifest, targetIndex: number): void {
    while (true) {
      const statement = this.reader.nextStatement();
      if (statement === undefined) {
        throw PrayError.parse(PARSE_CONTEXT, "missing 'end' for target block");
      }
      if (statement === "end") {
        return;
      }
      const target = manifest.targets[targetIndex];
      if (!target) {
        throw PrayError.manifest("target index out of range");
      }
      applyTargetStatement(manifest, target, statement);
    }
  }
}
