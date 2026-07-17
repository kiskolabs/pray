import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import {
  createServer,
  type IncomingMessage,
  type ServerResponse,
} from "node:http";
import { extname, join, resolve } from "node:path";
import { PrayError } from "../errors.js";

export function runServer(options: {
  root: string;
  host?: string;
  port?: number;
}): Promise<void> {
  const root = resolve(options.root);
  const host = options.host ?? "127.0.0.1";
  const port = options.port ?? 7429;

  return new Promise((resolvePromise, reject) => {
    const server = createServer((request, response) => {
      try {
        handleRequest(root, request, response);
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        writeResponse(response, 500, "text/plain", message);
      }
    });
    server.listen(port, host, () => {
      process.stdout.write(`Serving ${root} on http://${host}:${port}\n`);
    });
    server.on("error", reject);
    process.on("SIGINT", () => {
      server.close(() => resolvePromise());
    });
  });
}

function handleRequest(
  root: string,
  request: IncomingMessage,
  response: ServerResponse,
): void {
  const url = new URL(request.url ?? "/", "http://localhost");
  const method = request.method ?? "GET";
  const path = url.pathname;

  if (method === "PUT") {
    handlePut(root, path, request, response);
    return;
  }

  if (method !== "GET") {
    writeResponse(response, 405, "text/plain", "method not allowed");
    return;
  }

  if (path === "/") {
    writeResponse(
      response,
      200,
      "text/html",
      `<h1>Pray distribution</h1><p>Root: ${root}</p>`,
    );
    return;
  }

  const filePath = resolve(root, path.replace(/^\//, ""));
  if (!filePath.startsWith(root) || !existsSync(filePath)) {
    writeResponse(response, 404, "text/plain", "not found");
    return;
  }

  const body = readFileSync(filePath);
  writeResponse(response, 200, contentTypeFor(filePath), body);
}

function handlePut(
  root: string,
  path: string,
  request: IncomingMessage,
  response: ServerResponse,
): void {
  const chunks: Buffer[] = [];
  request.on("data", (chunk) => chunks.push(Buffer.from(chunk)));
  request.on("end", () => {
    const filePath = resolve(root, path.replace(/^\//, ""));
    if (!filePath.startsWith(root)) {
      writeResponse(response, 403, "text/plain", "forbidden");
      return;
    }
    mkdirSync(join(filePath, ".."), { recursive: true });
    writeFileSync(filePath, Buffer.concat(chunks));
    writeResponse(response, 200, "text/plain", "ok");
  });
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
): void {
  const buffer = typeof body === "string" ? Buffer.from(body, "utf8") : body;
  response.writeHead(status, {
    "Content-Type": contentType,
    "Content-Length": buffer.length,
    Connection: "close",
  });
  response.end(buffer);
}

export function runStdioRpc(): never {
  throw PrayError.unsupported("serve --stdio requires SSH RPC support");
}
