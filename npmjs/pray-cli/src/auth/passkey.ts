import { createPrivateKey, sign } from "node:crypto";
import { readFileSync } from "node:fs";
import { PrayError } from "../errors.js";
import { httpPostJson, joinUrl } from "../http/client.js";
import { persistSession, type SessionFile } from "./session.js";

interface PasskeyChallengeResponse {
  challenge_id: string;
  challenge: string;
}

interface PasskeyLoginResponse {
  email: string;
  token: string;
}

export async function loginWithPasskey(
  serverUrl: string,
  credentialId: string,
  privateKeyPath: string,
  sessionRoot: string,
): Promise<SessionFile> {
  const base = trimTrailingSlash(serverUrl);
  const challenge = await postJson<PasskeyChallengeResponse>(
    joinUrl(base, "v1/auth/passkeys/challenge"),
    { credential_id: credentialId },
  );
  const privateKeyBytes = readFileSync(privateKeyPath);
  if (privateKeyBytes.length !== 32) {
    throw PrayError.unsupported("passkey private key must be 32 raw bytes");
  }
  const privateKey = ed25519PrivateKeyFromSeed(privateKeyBytes);
  const signature = sign(
    null,
    Buffer.from(challenge.challenge, "utf8"),
    privateKey,
  ).toString("base64");
  const response = await postJson<PasskeyLoginResponse>(
    joinUrl(base, "v1/auth/passkeys/login"),
    {
      credential_id: credentialId,
      challenge_id: challenge.challenge_id,
      signature,
    },
  );
  return persistSession(sessionRoot, {
    server_url: serverUrl,
    email: response.email,
    token: response.token,
    kind: "passkey",
  });
}

function ed25519PrivateKeyFromSeed(seed: Buffer) {
  const prefix = Buffer.from("302e020100300506032b657004220420", "hex");
  return createPrivateKey({
    key: Buffer.concat([prefix, seed]),
    format: "der",
    type: "pkcs8",
  });
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
