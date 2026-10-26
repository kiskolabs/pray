export function applyByteRange(
  status: number,
  body: Buffer,
  rangeHeader: string | undefined,
): { status: number; body: Buffer; contentRange?: string } {
  if (!rangeHeader) {
    return { status, body };
  }
  if (status !== 200 || body.length === 0) {
    return { status, body };
  }
  const parsed = parseByteRange(rangeHeader, body.length);
  if (!parsed) {
    return {
      status: 416,
      body: Buffer.alloc(0),
      contentRange: `bytes */${body.length}`,
    };
  }
  return {
    status: 206,
    body: body.subarray(parsed.start, parsed.end + 1),
    contentRange: `bytes ${parsed.start}-${parsed.end}/${body.length}`,
  };
}

function parseByteRange(
  header: string,
  total: number,
): { start: number; end: number } | undefined {
  if (!header.startsWith("bytes=")) {
    return undefined;
  }
  const [startText, endText] = header.slice("bytes=".length).split("-", 2);
  if (startText === "" || endText === undefined || endText === "") {
    return undefined;
  }
  const start = Number(startText);
  const end = Number(endText);
  if (
    !Number.isInteger(start) ||
    !Number.isInteger(end) ||
    start > end ||
    end >= total
  ) {
    return undefined;
  }
  return { start, end };
}
