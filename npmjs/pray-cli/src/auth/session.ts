import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { PrayError } from "../errors.js";

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

export function sessionFilePath(root: string): string {
  return join(root, ".pray", "session.json");
}

export function persistSession(
  root: string,
  session: SessionFile,
): SessionFile {
  const path = sessionFilePath(root);
  mkdirSync(join(path, ".."), { recursive: true });
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
  writeFileSync(path, `${JSON.stringify(document, null, 2)}\n`, "utf8");
  return session;
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
