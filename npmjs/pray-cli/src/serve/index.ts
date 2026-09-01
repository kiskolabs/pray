import { randomUUID } from "node:crypto";
import {
  existsSync,
  mkdirSync,
  readFileSync,
  statSync,
  writeFileSync,
} from "node:fs";
import {
  createServer,
  type IncomingMessage,
  type ServerResponse,
} from "node:http";
import { extname, isAbsolute, join, relative, resolve } from "node:path";
import type { Readable } from "node:stream";
import { PrayError } from "../errors.js";
import {
  MAX_HTTP_RESPONSE_BYTES,
  MAX_SERVE_BODY_BYTES,
  MAX_SERVE_CONCURRENT_CONNECTIONS,
  MAX_SERVE_HEADER_BYTES,
  SERVE_SOCKET_TIMEOUT_MILLISECONDS,
} from "../resource-limits.js";

export function runServer(options: {
  root: string;
  host?: string;
  port?: number;
}): Promise<void> {
  const root = resolve(options.root);
  const host = options.host ?? "127.0.0.1";
  const port = options.port ?? 7429;

  return new Promise((resolvePromise, reject) => {
    const allowWrites = isLoopbackHost(host);
    const server = createServer(
      { maxHeaderSize: MAX_SERVE_HEADER_BYTES },
      (request, response) => {
        const requestId = randomUUID();
        void handleRequest(
          root,
          allowWrites,
          requestId,
          request,
          response,
        ).catch((error: unknown) => {
          const message =
            error instanceof Error ? error.message : String(error);
          process.stderr.write(
            `${JSON.stringify({ event: "serve_request_failed", request_id: requestId, message })}\n`,
          );
          writeResponse(
            response,
            500,
            "text/plain",
            "internal error",
            requestId,
          );
        });
      },
    );
    server.maxConnections = MAX_SERVE_CONCURRENT_CONNECTIONS;
    server.requestTimeout = SERVE_SOCKET_TIMEOUT_MILLISECONDS;
    server.headersTimeout = SERVE_SOCKET_TIMEOUT_MILLISECONDS;
    server.listen(port, host, () => {
      process.stdout.write(`Serving ${root} on http://${host}:${port}\n`);
    });
    server.on("error", reject);
    process.on("SIGINT", () => {
      server.close(() => resolvePromise());
    });
  });
}

async function handleRequest(
  root: string,
  allowWrites: boolean,
  requestId: string,
  request: IncomingMessage,
  response: ServerResponse,
): Promise<void> {
  const url = new URL(request.url ?? "/", "http://localhost");
  const method = request.method ?? "GET";
  const path = url.pathname;

  if (method === "PUT") {
    await handlePut(root, path, allowWrites, requestId, request, response);
    return;
  }

  if (method !== "GET") {
    writeResponse(response, 405, "text/plain", "method not allowed", requestId);
    return;
  }

  if (path === "/") {
    writeResponse(
      response,
      200,
      "text/html",
      "<h1>Pray distribution</h1>",
      requestId,
    );
    return;
  }

  if (path === "/health") {
    writeResponse(response, 200, "text/plain", "ok", requestId);
    return;
  }

  const filePath = containedPath(root, path);
  if (!filePath || !existsSync(filePath)) {
    writeResponse(response, 404, "text/plain", "not found", requestId);
    return;
  }

  if (statSync(filePath).size > MAX_HTTP_RESPONSE_BYTES) {
    writeResponse(
      response,
      413,
      "text/plain",
      "file exceeds server limit",
      requestId,
    );
    return;
  }
  const body = readFileSync(filePath);
  writeResponse(response, 200, contentTypeFor(filePath), body, requestId);
}

async function handlePut(
  root: string,
  path: string,
  allowWrites: boolean,
  requestId: string,
  request: IncomingMessage,
  response: ServerResponse,
): Promise<void> {
  if (!allowWrites) {
    writeResponse(response, 403, "text/plain", "forbidden", requestId);
    return;
  }
  const filePath = containedPath(root, path);
  if (!filePath) {
    writeResponse(response, 403, "text/plain", "forbidden", requestId);
    return;
  }
  let body: Buffer;
  try {
    body = await readRequestBody(request);
  } catch (error) {
    if (error instanceof PrayError) {
      writeResponse(response, 413, "text/plain", error.message, requestId);
      return;
    }
    throw error;
  }
  mkdirSync(join(filePath, ".."), { recursive: true });
  writeFileSync(filePath, body);
  writeResponse(response, 200, "text/plain", "ok", requestId);
}

export async function readRequestBody(request: Readable): Promise<Buffer> {
  const chunks: Buffer[] = [];
  let length = 0;
  for await (const chunk of request) {
    const bytes = Buffer.from(chunk);
    length += bytes.byteLength;
    if (length > MAX_SERVE_BODY_BYTES) {
      throw PrayError.resolution(
        `request body exceeds ${MAX_SERVE_BODY_BYTES} bytes`,
      );
    }
    chunks.push(bytes);
  }
  return Buffer.concat(chunks, length);
}

function containedPath(root: string, requestPath: string): string | undefined {
  const candidate = resolve(root, requestPath.replace(/^\//, ""));
  const fromRoot = relative(root, candidate);
  if (fromRoot.startsWith("..") || isAbsolute(fromRoot)) return undefined;
  return candidate;
}

function isLoopbackHost(host: string): boolean {
  return host === "127.0.0.1" || host === "::1" || host === "localhost";
}

function contentTypeFor(path: string): string {
  switch (extname(path)) {
    case ".json":
      return "application/json";
    case ".praypkg":
      return "application/octet-stream";
    default:
      return "text/plain";
  }
}

function writeResponse(
  response: ServerResponse,
  status: number,
  contentType: string,
  body: string | Buffer,
  requestId: string,
): void {
  if (response.headersSent || response.writableEnded) return;
  const buffer = typeof body === "string" ? Buffer.from(body, "utf8") : body;
  response.writeHead(status, {
    "Content-Type": contentType,
    "Content-Length": buffer.length,
    Connection: "close",
    "X-Request-ID": requestId,
  });
  response.end(buffer);
}

export function runStdioRpc(): never {
  throw PrayError.unsupported("serve --stdio requires SSH RPC support");
}
