import { printlnStdout, printlnVerbose } from "../cli/output.js";
import type { ResolvedPackage } from "../resolve/types.js";

export type PublishStats = { published: number; unchanged: number };

export function emptyPublishStats(): PublishStats {
  return { published: 0, unchanged: 0 };
}

export function mergePublishStats(
  left: PublishStats,
  right: PublishStats,
): PublishStats {
  return {
    published: left.published + right.published,
    unchanged: left.unchanged + right.unchanged,
  };
}

export function reportPublished(
  packageEntry: ResolvedPackage,
  destName: string,
  target: string,
  artifactPath: string,
): void {
  printlnStdout(
    `published ${packageEntry.declaration.name} ${packageEntry.spec.version} -> ${destName} (${target})`,
  );
  printlnVerbose(`  artifact ${artifactPath}`);
}

export function reportUnchanged(
  packageEntry: ResolvedPackage,
  destName: string,
  target: string,
): void {
  printlnStdout(
    `unchanged ${packageEntry.declaration.name} ${packageEntry.spec.version} -> ${destName} (${target})`,
  );
  printlnVerbose("  skip reason: stored publish matches");
}

export function printPublishFooter(stats: PublishStats): void {
  if (stats.published === 0 && stats.unchanged === 0) {
    printlnStdout("nothing to publish");
    return;
  }
  printlnStdout(`${stats.published} published, ${stats.unchanged} unchanged`);
}
