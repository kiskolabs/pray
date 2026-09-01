import { PrayError } from "../errors.js";
import { MAX_HTTP_RESPONSE_BYTES } from "../resource-limits.js";

export async function readBoundedHttpBody(
  response: Response,
  maxBytes: number = MAX_HTTP_RESPONSE_BYTES,
): Promise<Buffer> {
  const declared = response.headers.get("content-length");
  if (declared !== null) {
    const length = Number(declared);
    if (Number.isFinite(length) && length > maxBytes) {
      throw PrayError.resolution(`HTTP response exceeds ${maxBytes} bytes`);
    }
  }

  if (response.body === null) {
    return Buffer.alloc(0);
  }

  const reader = response.body.getReader();
  const chunks: Buffer[] = [];
  let length = 0;
  for (;;) {
    const { done, value } = await reader.read();
    if (done) {
      break;
    }
    const bytes = Buffer.from(value);
    length += bytes.byteLength;
    if (length > maxBytes) {
      await reader.cancel();
      throw PrayError.resolution(`HTTP response exceeds ${maxBytes} bytes`);
    }
    chunks.push(bytes);
  }
  return Buffer.concat(chunks, length);
}
