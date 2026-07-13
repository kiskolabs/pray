import { PrayError } from "../errors.js";

export async function httpGet(url: string): Promise<Buffer> {
  const response = await fetch(url);
  if (!response.ok) {
    throw PrayError.resolution(
      `HTTP request failed for ${url}: ${response.status}`,
    );
  }
  return Buffer.from(await response.arrayBuffer());
}

export async function httpGetText(url: string): Promise<string> {
  return (await httpGet(url)).toString("utf8");
}

export async function httpPut(
  url: string,
  contentType: string,
  body: Buffer | string,
): Promise<void> {
  const response = await fetch(url, {
    method: "PUT",
    headers: { "Content-Type": contentType },
    body: typeof body === "string" ? body : new Uint8Array(body),
  });
  if (!response.ok) {
    throw PrayError.resolution(
      `HTTP upload failed for ${url}: ${response.status}`,
    );
  }
}

export async function httpPost(
  url: string,
  contentType: string,
  body: Buffer | string,
): Promise<void> {
  const response = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": contentType },
    body: typeof body === "string" ? body : new Uint8Array(body),
  });
  if (!response.ok) {
    throw PrayError.resolution(
      `HTTP request failed for ${url}: ${response.status}`,
    );
  }
}

export function joinUrl(base: string, path: string): string {
  const normalizedBase = base.endsWith("/") ? base : `${base}/`;
  const normalizedPath = path.startsWith("/") ? path.slice(1) : path;
  return new URL(normalizedPath, normalizedBase).toString();
}

export function isRemoteUrl(sourceUrl: string): boolean {
  return (
    sourceUrl.startsWith("http://") ||
    sourceUrl.startsWith("https://") ||
    sourceUrl.startsWith("pray+ssh://") ||
    sourceUrl.startsWith("ssh+pray://")
  );
}

export function isLocalSourceUrl(sourceUrl: string): boolean {
  return !isRemoteUrl(sourceUrl);
}
