export class ContentBuilder {
  private content = "";
  private nextLine = 1;

  nextLineNumber(): number {
    return this.nextLine;
  }

  appendLine(line: string): void {
    this.content += `${line}\n`;
    this.nextLine += 1;
  }

  appendEmptyLine(): void {
    this.content += "\n";
    this.nextLine += 1;
  }

  appendBody(body: string): void {
    const trimmed = body.replace(/\n+$/, "");
    if (trimmed.length === 0) {
      return;
    }
    for (const line of trimmed.split("\n")) {
      this.appendLine(line);
    }
  }

  finish(): string {
    while (this.content.endsWith("\n\n")) {
      this.content = this.content.slice(0, -1);
    }
    if (!this.content.endsWith("\n")) {
      this.content += "\n";
    }
    return this.content;
  }
}
