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

export class PrayError extends Error {
  readonly kind: PrayErrorKind;
  readonly detail?: string;

  constructor(kind: PrayErrorKind, message: string, detail?: string) {
    super(message);
    this.name = "PrayError";
    this.kind = kind;
    this.detail = detail;
  }

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

  static manifest(message: string): PrayError {
    return new PrayError("manifest", message);
  }

  static parse(kind: string, message: string): PrayError {
    return new PrayError("parse", `${kind}: ${message}`, kind);
  }

  static resolution(message: string): PrayError {
    return new PrayError("resolution", message);
  }

  static integrity(message: string): PrayError {
    return new PrayError("integrity", message);
  }

  static render(message: string): PrayError {
    return new PrayError("render", message);
  }

  static verify(message: string): PrayError {
    return new PrayError("verify", message);
  }

  static io(message: string): PrayError {
    return new PrayError("io", message);
  }

  static unsupported(message: string): PrayError {
    return new PrayError("unsupported", message);
  }

  static usage(message: string): PrayError {
    return new PrayError("usage", message);
  }
}
