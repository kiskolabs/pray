import { readFileSync } from "node:fs";
import { connect } from "node:net";
import { PrayError } from "../errors.js";
import { httpPostJson, joinUrl } from "../http/client.js";
import { persistSession, type SessionFile } from "./session.js";

interface SshKeyChallengeResponse {
  challenge_id: string;
  challenge: string;
  fingerprint: string;
}

interface SshKeyLoginResponse {
  email: string;
  token: string;
}

export async function loginWithSshAgent(
  serverUrl: string,
  publicKeyPath: string,
  sessionRoot: string,
): Promise<SessionFile> {
  const publicKey = readFileSync(publicKeyPath, "utf8").trim();
  const base = trimTrailingSlash(serverUrl);
  const challenge = await postJson<SshKeyChallengeResponse>(
    joinUrl(base, "v1/auth/ssh-keys/challenge"),
    { public_key: publicKey },
  );
  const signature = await sshAgentSign(
    publicKey,
    Buffer.from(challenge.challenge, "utf8"),
  );
  const response = await postJson<SshKeyLoginResponse>(
    joinUrl(base, "v1/auth/ssh-keys/login"),
    {
      public_key: publicKey,
      challenge_id: challenge.challenge_id,
      signature,
    },
  );
  return persistSession(sessionRoot, {
    server_url: serverUrl,
    email: response.email,
    token: response.token,
    kind: "ssh_key",
    signer_fingerprint: challenge.fingerprint,
  });
}

async function sshAgentSign(
  publicKey: string,
  message: Buffer,
): Promise<string> {
  const agentSocket = process.env.SSH_AUTH_SOCK;
  if (!agentSocket) {
    throw PrayError.unsupported("SSH_AUTH_SOCK is not set");
  }
  const rawKeyBytes = parseSshEd25519PublicKey(publicKey);
  const publicKeyBlob = Buffer.concat([
    sshString(Buffer.from("ssh-ed25519")),
    sshString(rawKeyBytes),
  ]);
  const payload = Buffer.concat([
    sshString(publicKeyBlob),
    sshString(message),
    Buffer.from([0, 0, 0, 0]),
  ]);
  const response = await agentRequest(agentSocket, 13, payload);
  if (response.messageType !== 14) {
    throw PrayError.resolution(
      `ssh agent returned unexpected message type: ${response.messageType}`,
    );
  }
  const signatureBlob = readSshString(response.payload, 0).value;
  return parseSshSignatureBlob(signatureBlob);
}

function parseSshEd25519PublicKey(publicKey: string): Buffer {
  const fields = publicKey.split(/\s+/);
  const algorithm = fields[0];
  const keyValue = fields[1];
  if (!algorithm || !keyValue) {
    throw PrayError.unsupported("public key must include an algorithm and key");
  }
  if (algorithm !== "ssh-ed25519") {
    throw PrayError.unsupported(
      `unsupported public key algorithm: ${algorithm}`,
    );
  }
  const blob = Buffer.from(keyValue, "base64");
  const algorithmField = readSshString(blob, 0);
  if (algorithmField.value.toString("utf8") !== "ssh-ed25519") {
    throw PrayError.parse(
      "public key",
      "ed25519 public key blob must start with ssh-ed25519",
    );
  }
  const keyBytes = readSshString(blob, algorithmField.nextOffset).value;
  if (keyBytes.length !== 32) {
    throw PrayError.parse("public key", "ed25519 public key must be 32 bytes");
  }
  return keyBytes;
}

function parseSshSignatureBlob(signatureBlob: Buffer): string {
  const algorithm = readSshString(signatureBlob, 0);
  if (algorithm.value.toString("utf8") !== "ssh-ed25519") {
    throw PrayError.unsupported(
      `unsupported ssh signature algorithm: ${algorithm.value.toString("utf8")}`,
    );
  }
  const signature = readSshString(signatureBlob, algorithm.nextOffset).value;
  return signature.toString("base64");
}

function agentRequest(
  socketPath: string,
  messageType: number,
  payload: Buffer,
): Promise<{ messageType: number; payload: Buffer }> {
  return new Promise((resolve, reject) => {
    const socket = connect(socketPath);
    const chunks: Buffer[] = [];
    socket.on("error", (error) => reject(error));
    socket.on("data", (chunk) => chunks.push(chunk));
    socket.on("end", () => {
      try {
        resolve(parseAgentResponse(Buffer.concat(chunks)));
      } catch (error) {
        reject(error);
      }
    });
    const body = Buffer.concat([Buffer.from([messageType]), payload]);
    const frame = Buffer.alloc(4 + body.length);
    frame.writeUInt32BE(body.length, 0);
    body.copy(frame, 4);
    socket.end(frame);
  });
}

function parseAgentResponse(buffer: Buffer): {
  messageType: number;
  payload: Buffer;
} {
  if (buffer.length < 5) {
    throw PrayError.resolution("empty ssh agent response");
  }
  const length = buffer.readUInt32BE(0);
  const body = buffer.subarray(4, 4 + length);
  return { messageType: body[0]!, payload: body.subarray(1) };
}

function sshString(bytes: Buffer): Buffer {
  const buffer = Buffer.alloc(4 + bytes.length);
  buffer.writeUInt32BE(bytes.length, 0);
  bytes.copy(buffer, 4);
  return buffer;
}

function readSshString(
  buffer: Buffer,
  offset: number,
): { value: Buffer; nextOffset: number } {
  if (buffer.length < offset + 4) {
    throw PrayError.resolution("truncated ssh field");
  }
  const length = buffer.readUInt32BE(offset);
  const start = offset + 4;
  const end = start + length;
  if (buffer.length < end) {
    throw PrayError.resolution("truncated ssh agent response");
  }
  return { value: buffer.subarray(start, end), nextOffset: end };
}

async function postJson<T>(url: string, body: unknown): Promise<T> {
  const responseBody = await httpPostJson(
    url,
    "application/json",
    JSON.stringify(body),
  );
  try {
    return JSON.parse(responseBody.toString("utf8")) as T;
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw PrayError.parse("auth response", message);
  }
}

function trimTrailingSlash(value: string): string {
  return value.replace(/\/+$/, "");
}
