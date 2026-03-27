export function refillSelect(selectEl, values, current) {
  const optionValues = ["ALL", ...values];
  selectEl.innerHTML = "";

  for (const value of optionValues) {
    const option = document.createElement("option");
    option.value = value;
    option.textContent = value;
    selectEl.appendChild(option);
  }

  selectEl.value = optionValues.includes(current) ? current : "ALL";
}

export function renderStats(ui, state) {
  ui.directoryText.textContent = state.directoryName || "Not selected";
  ui.modeText.textContent = state.refreshMode;
  ui.countText.textContent = String(state.entries.length);
  ui.filteredCountText.textContent = String(state.filteredEntries.length);
  ui.errorCountText.textContent = String(state.parseErrors);
  ui.refreshText.textContent = state.lastRefreshAt
    ? state.lastRefreshAt.toLocaleTimeString()
    : "-";
}

export function renderList(ui, state, keyword, onSelect) {
  ui.logList.innerHTML = "";
  if (state.filteredEntries.length === 0) {
    const empty = document.createElement("div");
    empty.className = "empty";
    empty.textContent = state.directoryHandle
      ? "No logs match current filters."
      : "Choose a directory to load core logs.";
    ui.logList.appendChild(empty);
    return;
  }

  const tokens = tokenize(keyword);
  const fragment = document.createDocumentFragment();

  state.filteredEntries.forEach((entry, index) => {
    const row = ui.rowTemplate.content.firstElementChild.cloneNode(true);
    row.style.setProperty("--idx", String(Math.min(index, 24)));

    const badge = row.querySelector(".badge");
    badge.textContent = entry.level;
    badge.classList.add(entry.level.toLowerCase());

    row.querySelector(".target").textContent = entry.target;
    row.querySelector(".time").textContent =
      entry.timestamp || "(no timestamp)";
    row.querySelector(".row-foot").textContent =
      `${entry.fileName}:${entry.lineNumber}`;

    const messageEl = row.querySelector(".message");
    appendHighlighted(messageEl, entry.message, tokens);

    if (entry.id === state.selectedId) {
      row.classList.add("selected");
    }

    row.addEventListener("click", () => onSelect(entry.id));
    fragment.appendChild(row);
  });

  ui.logList.appendChild(fragment);
}

export function renderDetail(ui, state, keyword) {
  ui.jsonTree.innerHTML = "";
  const entry = state.filteredEntries.find(
    (item) => item.id === state.selectedId,
  );
  if (!entry) {
    ui.detailMeta.textContent = "Select one row from Log Stream.";
    return;
  }

  ui.detailMeta.textContent = `${entry.timestamp || "(no timestamp)"}  ${entry.level}  ${entry.target}  ${entry.fileName}:${entry.lineNumber}`;
  const root = createJsonNode("$", entry.raw, "$", tokenize(keyword), 0);
  ui.jsonTree.appendChild(root);
}

function createJsonNode(key, value, path, tokens, depth) {
  const node = document.createElement("div");
  node.className = "json-node";

  if (!isObject(value) && !Array.isArray(value)) {
    const line = document.createElement("div");
    const keyEl = document.createElement("span");
    keyEl.className = "json-key";
    keyEl.textContent = `${key}: `;
    line.appendChild(keyEl);

    const valueEl = document.createElement("span");
    valueEl.className = value === null ? "json-null" : `json-${typeof value}`;
    if (typeof value === "string") {
      appendHighlighted(valueEl, JSON.stringify(value), tokens);
    } else {
      valueEl.textContent = JSON.stringify(value);
    }

    line.appendChild(valueEl);
    node.appendChild(line);
    return node;
  }

  const details = document.createElement("details");
  details.open = depth < 2;

  const summary = document.createElement("summary");
  const keyEl = document.createElement("span");
  keyEl.className = "json-key";
  const size = Array.isArray(value) ? value.length : Object.keys(value).length;
  keyEl.textContent = `${key}: ${Array.isArray(value) ? `[${size}]` : `{${size}}`} `;
  summary.appendChild(keyEl);

  const pathEl = document.createElement("span");
  pathEl.className = "row-foot";
  pathEl.textContent = path;
  summary.appendChild(pathEl);

  details.appendChild(summary);

  const children = document.createElement("div");
  if (Array.isArray(value)) {
    value.forEach((item, index) => {
      children.appendChild(
        createJsonNode(
          String(index),
          item,
          `${path}[${index}]`,
          tokens,
          depth + 1,
        ),
      );
    });
  } else {
    for (const [childKey, childValue] of Object.entries(value)) {
      const childPath = path === "$" ? `$.${childKey}` : `${path}.${childKey}`;
      children.appendChild(
        createJsonNode(childKey, childValue, childPath, tokens, depth + 1),
      );
    }
  }

  details.appendChild(children);
  node.appendChild(details);
  return node;
}

function appendHighlighted(container, text, tokens) {
  if (!tokens.length) {
    container.textContent = text;
    return;
  }

  const escapedTokens = tokens.map((token) =>
    token.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"),
  );
  const regex = new RegExp(`(${escapedTokens.join("|")})`, "ig");
  const parts = text.split(regex);

  for (const part of parts) {
    if (!part) {
      continue;
    }

    const hit = escapedTokens.some(
      (token) => part.toLowerCase() === token.toLowerCase(),
    );
    if (hit) {
      const mark = document.createElement("mark");
      mark.textContent = part;
      container.appendChild(mark);
    } else {
      container.appendChild(document.createTextNode(part));
    }
  }
}

function tokenize(value) {
  return value.toLowerCase().split(/\s+/).filter(Boolean);
}

function isObject(value) {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
