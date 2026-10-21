let verboseLevel = 0;

export function configureOutput(quiet: boolean, verbose: number): void {
  if (quiet) {
    process.env.PRAY_QUIET = "1";
  }
  verboseLevel = verbose;
  if (verbose > 0) {
    process.env.PRAY_VERBOSE = String(verbose);
  }
}

export function quiet(): boolean {
  const value = process.env.PRAY_QUIET;
  return value !== undefined && value !== "0";
}

export function verbose(): boolean {
  if (verboseLevel > 0) {
    return true;
  }
  const level = Number.parseInt(process.env.PRAY_VERBOSE ?? "0", 10);
  return Number.isFinite(level) && level > 0;
}

export function printlnStdout(message: string): void {
  if (quiet()) {
    return;
  }
  process.stdout.write(`${message}\n`);
}

export function printlnVerbose(message: string): void {
  if (quiet() || !verbose()) {
    return;
  }
  process.stdout.write(`${message}\n`);
}
