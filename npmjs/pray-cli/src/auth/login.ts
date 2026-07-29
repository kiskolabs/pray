import { PrayError } from "../errors.js";
import { loginWithPasskey } from "./passkey.js";
import { loginWithSshAgent } from "./ssh-agent.js";

export interface LoginOptions {
  servers: string[];
  email: string;
  credentialId?: string;
  passkeyKey?: string;
  publicKey?: string;
  sshAgent: boolean;
}

export function parseLoginArguments(argumentsList: string[]): LoginOptions {
  const servers: string[] = [];
  let email: string | undefined;
  let credentialId: string | undefined;
  let passkeyKey: string | undefined;
  let publicKey: string | undefined;
  let sshAgent = false;

  for (let index = 0; index < argumentsList.length; index += 1) {
    const argument = argumentsList[index]!;
    const next = (): string => {
      const value = argumentsList[index + 1];
      if (!value) {
        throw PrayError.unsupported(`login requires a value after ${argument}`);
      }
      index += 1;
      return value;
    };
    switch (argument) {
      case "--server":
        servers.push(next());
        break;
      case "--email":
        email = next();
        break;
      case "--credential-id":
        credentialId = next();
        break;
      case "--passkey-key":
        passkeyKey = next();
        break;
      case "--public-key":
        publicKey = next();
        break;
      case "--ssh-agent":
        sshAgent = true;
        break;
      default:
        if (argument.startsWith("--")) {
          throw PrayError.unsupported(`unknown login flag: ${argument}`);
        }
        throw PrayError.unsupported(`unexpected login argument: ${argument}`);
    }
  }

  if (servers.length === 0) {
    throw PrayError.unsupported("login requires at least one --server URL");
  }
  if (!email) {
    throw PrayError.unsupported("login requires --email ADDRESS");
  }
  const passkeyMode = passkeyKey !== undefined;
  if (passkeyMode === sshAgent || (!passkeyMode && !publicKey && !sshAgent)) {
    throw PrayError.unsupported(
      "login requires exactly one authentication mode",
    );
  }
  if (passkeyMode && !credentialId) {
    throw PrayError.unsupported("passkey login requires --credential-id");
  }
  if (sshAgent && !publicKey) {
    throw PrayError.unsupported("ssh-agent login requires --public-key");
  }

  return {
    servers,
    email,
    credentialId,
    passkeyKey,
    publicKey,
    sshAgent,
  };
}

export async function runLoginCommand(
  argumentsList: string[],
  sessionRoot: string,
): Promise<void> {
  const options = parseLoginArguments(argumentsList);
  for (const server of options.servers) {
    const session = options.passkeyKey
      ? await loginWithPasskey(
          server,
          options.credentialId!,
          options.passkeyKey,
          sessionRoot,
        )
      : await loginWithSshAgent(server, options.publicKey!, sessionRoot);
    if (session.email !== options.email) {
      throw PrayError.resolution(
        `login completed for ${session.email} but ${options.email} was requested`,
      );
    }
    process.stdout.write(
      `logged in as ${session.email} via ${session.kind} on ${server}\n`,
    );
  }
}
