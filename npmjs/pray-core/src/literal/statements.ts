import { isBalanced } from "./split.js";
import { SurfaceStatementReader } from "./statement-surface.js";

export class StatementReader {
  private cursor = 0;
  private readonly surface = new SurfaceStatementReader();

  constructor(private readonly lines: readonly string[]) {}

  nextStatement(): string | undefined {
    const pending = this.surface.next();
    if (pending !== undefined) {
      return pending;
    }
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
      this.surface.pushRaw(statement);
      const normalized = this.surface.next();
      if (normalized !== undefined) {
        return normalized;
      }
    }
    return undefined;
  }
}
