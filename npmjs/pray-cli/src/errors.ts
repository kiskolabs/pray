export type PrayErrorKind =
  | "manifest"
  | "parse"
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
      case "verify":
        return 2;
      case "unsupported":
        return 3;
      default:
        return 1;
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
}
