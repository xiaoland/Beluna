const LOG_FILE_PATTERN = /^core\.log\.(\d{4}-\d{2}-\d{2})\.(\d+)$/;

export async function listLogFiles(directoryHandle) {
  const records = [];
  for await (const [name, handle] of directoryHandle.entries()) {
    if (handle.kind !== "file") {
      continue;
    }

    const match = name.match(LOG_FILE_PATTERN);
    if (!match) {
      continue;
    }

    records.push({
      name,
      date: match[1],
      sequence: Number.parseInt(match[2], 10),
      handle,
    });
  }

  records.sort((a, b) => {
    if (a.date === b.date) {
      return a.sequence - b.sequence;
    }
    return a.date.localeCompare(b.date);
  });
  return records;
}

export function parseChunk(fileName, chunk, fileState, onEntry, onError) {
  const combined = fileState.remainder + chunk;
  const lines = combined.split(/\r?\n/);
  fileState.remainder = lines.pop() ?? "";

  for (const line of lines) {
    fileState.line += 1;
    if (!line.trim()) {
      continue;
    }

    try {
      const parsed = JSON.parse(line);
      onEntry(makeEntry(parsed, fileName, fileState.line));
    } catch {
      onError();
    }
  }
}

export function applyFilters(entries, filters) {
  return entries
    .filter((entry) => {
      if (filters.level !== "ALL" && entry.level !== filters.level) {
        return false;
      }
      if (filters.target !== "ALL" && entry.target !== filters.target) {
        return false;
      }
      if (
        filters.startMs !== null &&
        (entry.timestampMs === null || entry.timestampMs < filters.startMs)
      ) {
        return false;
      }
      if (
        filters.endMs !== null &&
        (entry.timestampMs === null || entry.timestampMs > filters.endMs)
      ) {
        return false;
      }
      if (
        filters.tokens.length > 0 &&
        !filters.tokens.every((token) => entry.searchText.includes(token))
      ) {
        return false;
      }
      return true;
    })
    .sort((a, b) => {
      if (
        a.timestampMs !== null &&
        b.timestampMs !== null &&
        a.timestampMs !== b.timestampMs
      ) {
        return b.timestampMs - a.timestampMs;
      }
      return b.id.localeCompare(a.id);
    });
}

export function makeEntry(raw, fileName, lineNumber) {
  const fields = isObject(raw.fields) ? raw.fields : {};
  const message =
    typeof fields.message === "string"
      ? fields.message
      : typeof raw.message === "string"
        ? raw.message
        : "(no message)";
  const timestamp = typeof raw.timestamp === "string" ? raw.timestamp : "";
  const parsedTimestamp = Date.parse(timestamp);

  return {
    id: `${fileName}:${lineNumber}`,
    fileName,
    lineNumber,
    level: typeof raw.level === "string" ? raw.level : "UNKNOWN",
    target: typeof raw.target === "string" ? raw.target : "unknown",
    message,
    timestamp,
    timestampMs: Number.isNaN(parsedTimestamp) ? null : parsedTimestamp,
    raw,
    searchText: buildSearchText(raw).toLowerCase(),
  };
}

function buildSearchText(value) {
  const stack = [value];
  const parts = [];
  while (stack.length > 0) {
    const next = stack.pop();
    if (next === null || next === undefined) {
      continue;
    }

    if (Array.isArray(next)) {
      for (const item of next) {
        stack.push(item);
      }
      continue;
    }

    if (isObject(next)) {
      for (const [key, item] of Object.entries(next)) {
        parts.push(String(key));
        stack.push(item);
      }
      continue;
    }

    parts.push(String(next));
  }

  return parts.join(" ");
}

function isObject(value) {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
