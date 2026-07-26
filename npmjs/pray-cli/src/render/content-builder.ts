export class ContentBuilder {
  private content = "";

  nextLineNumber(): number {
    return this.content.split("\n").length;
  }

  appendLine(line: string): void {
    this.content += `${line}\n`;
  }

  appendEmptyLine(): void {
    this.content += "\n";
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
