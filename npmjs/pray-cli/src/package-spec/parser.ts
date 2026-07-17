import { PrayError } from "../errors.js";
import { stringFromLiteral } from "../literal/call-parser.js";
import { prepareParserLines } from "../literal/lines.js";
import { StatementReader } from "../literal/statements.js";
import {
  arrayOfStrings,
  parseDependency,
  parseExports,
  parseSkills,
  parseStringMap,
  parseTemplates,
} from "./parse-maps.js";
import { canonicalPackageSpec, type PackageSpec } from "./types.js";

const PARSE_CONTEXT = "prayspec";

export function parsePackageSpecText(text: string): PackageSpec {
  const parser = new BlockParser(prepareParserLines(text));
  return parser.parseRoot();
}

class BlockParser {
  private readonly reader: StatementReader;

  constructor(lines: string[]) {
    this.reader = new StatementReader(lines);
  }

  parseRoot(): PackageSpec {
    const start = this.reader.nextStatement();
    if (!start?.startsWith("Package::Specification.new")) {
      throw PrayError.parse(
        PARSE_CONTEXT,
        "expected Package::Specification.new",
      );
    }

    const spec: PackageSpec = {
      name: "",
      version: "",
      authors: [],
      files: [],
      exports: new Map(),
      skills: new Map(),
      templates: new Map(),
      adapters: new Map(),
      targets: [],
      dependencies: [],
      metadata: new Map(),
    };

    while (true) {
      const statement = this.reader.nextStatement();
      if (statement === undefined) {
        throw PrayError.parse(PARSE_CONTEXT, "missing 'end'");
      }
      if (statement === "end") {
        return canonicalPackageSpec(spec);
      }
      this.applyStatement(spec, statement);
    }
  }

  private applyStatement(spec: PackageSpec, statement: string): void {
    if (statement.startsWith("spec.add_dependency ")) {
      spec.dependencies.push(
        parseDependency(statement.slice("spec.add_dependency ".length), false),
      );
      return;
    }
    if (statement.startsWith("spec.add_optional_dependency ")) {
      spec.dependencies.push(
        parseDependency(
          statement.slice("spec.add_optional_dependency ".length),
          true,
        ),
      );
      return;
    }
    if (statement.startsWith("spec.")) {
      const rest = statement.slice("spec.".length);
      const separator = rest.indexOf(" = ");
      if (separator === -1) {
        throw PrayError.parse(
          PARSE_CONTEXT,
          `unrecognized statement: ${statement}`,
        );
      }
      const field = rest.slice(0, separator).trim();
      const value = rest.slice(separator + 3).trim();
      this.applyAssignment(spec, field, value);
      return;
    }
    throw PrayError.parse(
      PARSE_CONTEXT,
      `unrecognized statement: ${statement}`,
    );
  }

  private applyAssignment(
    spec: PackageSpec,
    field: string,
    value: string,
  ): void {
    switch (field) {
      case "name":
        spec.name = stringFromLiteral(value, PARSE_CONTEXT);
        return;
      case "version":
        spec.version = stringFromLiteral(value, PARSE_CONTEXT);
        return;
      case "summary":
        spec.summary = stringFromLiteral(value, PARSE_CONTEXT);
        return;
      case "description":
        spec.description = stringFromLiteral(value, PARSE_CONTEXT);
        return;
      case "authors":
        spec.authors = arrayOfStrings(value);
        return;
      case "license":
        spec.license = stringFromLiteral(value, PARSE_CONTEXT);
        return;
      case "files":
        spec.files = arrayOfStrings(value);
        return;
      case "targets":
        spec.targets = arrayOfStrings(value);
        return;
      case "exports":
        spec.exports = parseExports(value);
        return;
      case "skills":
        spec.skills = parseSkills(value);
        return;
      case "templates":
        spec.templates = parseTemplates(value);
        return;
      case "adapters":
        spec.adapters = parseStringMap(value);
        return;
      default:
        throw PrayError.parse(
          PARSE_CONTEXT,
          `unsupported assignment: ${field}`,
        );
    }
  }
}
