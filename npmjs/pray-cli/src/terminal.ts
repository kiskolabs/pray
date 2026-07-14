function envTruthy(name: string): boolean {
  const value = process.env[name];
  if (!value) {
    return false;
  }
  const normalized = value.trim().toLowerCase();
  return normalized === "1" || normalized === "true" || normalized === "yes" || normalized === "on";
}

export function noColorRequested(): boolean {
  const value = process.env.NO_COLOR;
  return value !== undefined && value.length > 0;
}

export function colorEnabled(): boolean {
  if (envTruthy("PRAY_NO_COLOR") || noColorRequested()) {
    return false;
  }
  const term = process.env.TERM;
  return term === undefined || term !== "dumb";
}

export function noInputRequested(): boolean {
  return envTruthy("PRAY_NO_INPUT");
}
