export const TOP_LEVEL_COMMANDS = [
  "add",
  "apply",
  "clean",
  "confess",
  "drift",
  "explain",
  "format",
  "help",
  "init",
  "install",
  "list",
  "login",
  "manifest",
  "outdated",
  "package",
  "plan",
  "prayer",
  "publish",
  "remove",
  "render",
  "repo",
  "serve",
  "sync",
  "tree",
  "trust",
  "unlock",
  "update",
  "vendor",
  "verify",
  "version",
] as const;

export function unknownCommandMessage(command: string): string {
  let message = `unknown command: ${command}`;
  const suggestion = suggestCommand(command, TOP_LEVEL_COMMANDS);
  if (suggestion) {
    message += `\nDid you mean \`${suggestion}\`?`;
  }
  return message;
}

export function suggestCommand(
  input: string,
  candidates: readonly string[],
): string | undefined {
  const maximumDistance = input.length <= 3 ? 1 : 2;
  let best: { candidate: string; distance: number } | undefined;
  for (const candidate of candidates) {
    const distance = levenshteinDistance(input, candidate);
    if (distance > maximumDistance) {
      continue;
    }
    if (!best || distance < best.distance) {
      best = { candidate, distance };
    }
  }
  return best?.candidate;
}

function levenshteinDistance(left: string, right: string): number {
  const leftChars = [...left];
  const rightChars = [...right];
  const leftLength = leftChars.length;
  const rightLength = rightChars.length;
  if (leftLength === 0) {
    return rightLength;
  }
  if (rightLength === 0) {
    return leftLength;
  }

  let previousRow = Array.from({ length: rightLength + 1 }, (_, index) => index);
  let currentRow = Array.from({ length: rightLength + 1 }, () => 0);

  for (let leftIndex = 0; leftIndex < leftLength; leftIndex += 1) {
    currentRow[0] = leftIndex + 1;
    for (let rightIndex = 0; rightIndex < rightLength; rightIndex += 1) {
      const leftCharacter = leftChars[leftIndex];
      const rightCharacter = rightChars[rightIndex];
      if (leftCharacter === undefined || rightCharacter === undefined) {
        continue;
      }
      const substitutionCost = leftCharacter === rightCharacter ? 0 : 1;
      const deleteCost = (previousRow[rightIndex + 1] ?? 0) + 1;
      const insertCost = (currentRow[rightIndex] ?? 0) + 1;
      const substituteCost = (previousRow[rightIndex] ?? 0) + substitutionCost;
      currentRow[rightIndex + 1] = Math.min(deleteCost, insertCost, substituteCost);
    }
    [previousRow, currentRow] = [currentRow, previousRow];
  }

  return previousRow[rightLength] ?? 0;
}
