import {
  chmodSync,
  existsSync,
  mkdirSync,
  readFileSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { PrayError } from "../errors.js";
import { trustHome } from "../trust/store.js";

export interface SessionFile {
  server_url: string;
  email: string;
  token: string;
  kind: string;
  signer_fingerprint?: string;
}

type SessionDocument =
  | SessionFile
  | {
      sessions: SessionFile[];
    };

export function sessionFilePath(_root: string): string {
  return join(trustHome(), "session.json");
}

export function persistSession(
  root: string,
  session: SessionFile,
): SessionFile {
  const path = sessionFilePath(root);
  mkdirSync(join(path, ".."), { recursive: true });
  migrateLegacySession(root, path);
  const sessions = loadSessions(path) ?? [];
  const existingIndex = sessions.findIndex(
    (entry) => entry.server_url === session.server_url,
  );
  if (existingIndex >= 0) {
    sessions[existingIndex] = session;
  } else {
    sessions.push(session);
  }
  const document: SessionDocument =
    sessions.length === 1 ? sessions[0]! : { sessions };
  writeSessionDocument(path, document);
  return session;
}

function migrateLegacySession(root: string, path: string): void {
  const legacyPath = join(root, ".pray", "session.json");
  if (!existsSync(legacyPath) || legacyPath === path) return;
  const sessions = loadSessions(path) ?? [];
  for (const legacy of loadSessions(legacyPath) ?? []) {
    if (!sessions.some((entry) => entry.server_url === legacy.server_url)) {
      sessions.push(legacy);
    }
  }
  const document: SessionDocument =
    sessions.length === 1 ? sessions[0]! : { sessions };
  writeSessionDocument(path, document);
  rmSync(legacyPath);
}

function writeSessionDocument(path: string, document: SessionDocument): void {
  const temporaryPath = `${path}.tmp-${process.pid}`;
  writeFileSync(temporaryPath, `${JSON.stringify(document, null, 2)}\n`, {
    encoding: "utf8",
    mode: 0o600,
  });
  chmodSync(temporaryPath, 0o600);
  renameSync(temporaryPath, path);
}

export function loadSessions(path: string): SessionFile[] | undefined {
  if (!existsSync(path)) {
    return undefined;
  }
  try {
    const document = JSON.parse(readFileSync(path, "utf8")) as SessionDocument;
    if ("sessions" in document && Array.isArray(document.sessions)) {
      return document.sessions;
    }
    return [document as SessionFile];
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw PrayError.parse("session file", message);
  }
}
