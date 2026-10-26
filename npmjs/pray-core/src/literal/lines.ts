export function prepareParserLines(text: string): string[] {
  return text.split(/\r?\n/).map(prepareParserLine);
}

function prepareParserLine(line: string): string {
  return stripLineComment(line).trimEnd();
}

export function stripLineComment(line: string): string {
  let quote: string | undefined;
  let escaped = false;

  for (let index = 0; index < line.length; index += 1) {
    const character = line[index];
    if (quote) {
      if (escaped) {
        escaped = false;
      } else if (character === "\\") {
        escaped = true;
      } else if (character === quote) {
        quote = undefined;
      }
      continue;
    }

    if (character === '"' || character === "'") {
      quote = character;
      continue;
    }
    if (character === "#") {
      return line.slice(0, index);
    }
  }

  return line;
}
