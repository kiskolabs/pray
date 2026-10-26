/** Category of a {@link PrayError}, used for exit codes and formatting. */
export type PrayErrorKind =
  | "manifest"
  | "parse"
  | "usage"
  | "resolution"
  | "integrity"
  | "render"
  | "verify"
  | "io"
  | "unsupported";

/** Typed error raised by the Prayfile library and CLI. */
export class PrayError extends Error {
  /** Error category for exit codes and message formatting. */
  readonly kind: PrayErrorKind;
  /** Optional secondary detail (for example a parse sub-kind). */
  readonly detail?: string;

  constructor(kind: PrayErrorKind, message: string, detail?: string) {
    super(message);
    this.name = "PrayError";
    this.kind = kind;
    this.detail = detail;
  }

  /** Process exit code associated with this error kind. */
  exitCode(): number {
    switch (this.kind) {
      case "usage":
      case "parse":
        return 2;
      case "resolution":
        return 3;
      case "integrity":
        return 4;
      case "render":
        return 5;
      case "verify":
        return 6;
      case "unsupported":
        return 8;
      default:
        return 1;
    }
  }

  /** Human-readable message, with a prefix for usage and unsupported errors. */
  toString(): string {
    switch (this.kind) {
      case "usage":
        return `usage error: ${this.message}`;
      case "unsupported":
        return `unsupported feature: ${this.message}`;
      default:
        return this.message;
    }
  }

  /** Build a manifest-related error. */
  static manifest(message: string): PrayError {
    return new PrayError("manifest", message);
  }

  /** Build a parse error with a sub-kind detail. */
  static parse(kind: string, message: string): PrayError {
    return new PrayError("parse", `${kind}: ${message}`, kind);
  }

  /** Build a resolution error. */
  static resolution(message: string): PrayError {
    return new PrayError("resolution", message);
  }

  /** Build an integrity error. */
  static integrity(message: string): PrayError {
    return new PrayError("integrity", message);
  }

  /** Build a render error. */
  static render(message: string): PrayError {
    return new PrayError("render", message);
  }

  /** Build a verify error. */
  static verify(message: string): PrayError {
    return new PrayError("verify", message);
  }

  /** Build an I/O error. */
  static io(message: string): PrayError {
    return new PrayError("io", message);
  }

  /** Build an unsupported-feature error. */
  static unsupported(message: string): PrayError {
    return new PrayError("unsupported", message);
  }

  /** Build a CLI usage error. */
  static usage(message: string): PrayError {
    return new PrayError("usage", message);
  }
}
