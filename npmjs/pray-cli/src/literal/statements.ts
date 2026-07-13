import { isBalanced } from "./split.js";

export class StatementReader {
  private cursor = 0;

  constructor(private readonly lines: readonly string[]) {}

  nextStatement(): string | undefined {
    while (this.cursor < this.lines.length) {
      const line = this.lines[this.cursor];
      if (line === undefined) {
        break;
      }
      let statement = line.trim();
      this.cursor += 1;
      if (statement.length === 0) {
        continue;
      }
      while (
        !statement.endsWith(" do") &&
        statement !== "end" &&
        this.cursor < this.lines.length &&
        (statement.trimEnd().endsWith(",") || !isBalanced(statement))
      ) {
        const nextLine = this.lines[this.cursor];
        if (nextLine === undefined) {
          break;
        }
        const next = nextLine.trim();
        this.cursor += 1;
        if (next.length === 0) {
          continue;
        }
        statement = `${statement} ${next}`;
      }
      return statement;
    }
    return undefined;
  }
}
