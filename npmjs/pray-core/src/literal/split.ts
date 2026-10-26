export function splitTopLevel(input: string, separator: string): string[] {
  const output: string[] = [];
  let start = 0;
  let depth = 0;
  let quote: string | undefined;
  let escaped = false;

  for (let index = 0; index < input.length; index += 1) {
    const character = input[index];
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
    if (character === "[" || character === "{" || character === "(") {
      depth += 1;
      continue;
    }
    if (character === "]" || character === "}" || character === ")") {
      depth -= 1;
      continue;
    }
    if (character === separator && depth === 0) {
      output.push(input.slice(start, index).trim());
      start = index + 1;
    }
  }

  if (start < input.length) {
    output.push(input.slice(start).trim());
  }

  return output.filter((segment) => segment.length > 0);
}

export function findTopLevel(input: string, token: string): number | undefined {
  let depth = 0;
  let quote: string | undefined;
  let escaped = false;

  for (let index = 0; index < input.length; index += 1) {
    const character = input[index];
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
    if (character === "[" || character === "{" || character === "(") {
      depth += 1;
      continue;
    }
    if (character === "]" || character === "}" || character === ")") {
      depth -= 1;
      continue;
    }
    if (depth === 0 && input.startsWith(token, index)) {
      return index;
    }
  }

  return undefined;
}

export function isBalanced(input: string): boolean {
  let depth = 0;
  let quote: string | undefined;
  let escaped = false;

  for (const character of input) {
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
    if (character === "[" || character === "{" || character === "(") {
      depth += 1;
      continue;
    }
    if (character === "]" || character === "}" || character === ")") {
      depth -= 1;
    }
  }

  return depth === 0 && !quote;
}
