import { applyFilters, listLogFiles, parseChunk } from "./parser.js";
import {
  DEFAULT_MAX_ENTRIES,
  getMaxEntries,
  loadPersistedFilters,
  persistFilters,
} from "./preferences.js";
import { configureRefreshMode } from "./refresh.js";
import { refillSelect, renderDetail, renderList, renderStats } from "./view.js";

const POLL_MS = 2000;

const state = {
  directoryHandle: null,
  directoryName: "",
  entries: [],
  filteredEntries: [],
  selectedId: "",
  parseErrors: 0,
  fileStates: new Map(),
  loading: false,
  queuedReset: false,
  refreshMode: "idle",
  observer: null,
  pollingId: null,
  lastRefreshAt: null,
};

const ui = {
  chooseDirBtn: document.querySelector("#chooseDirBtn"),
  refreshBtn: document.querySelector("#refreshBtn"),
  clearFiltersBtn: document.querySelector("#clearFiltersBtn"),
  autoRefreshToggle: document.querySelector("#autoRefreshToggle"),
  keywordInput: document.querySelector("#keywordInput"),
  levelSelect: document.querySelector("#levelSelect"),
  targetSelect: document.querySelector("#targetSelect"),
  startInput: document.querySelector("#startInput"),
  endInput: document.querySelector("#endInput"),
  maxEntriesInput: document.querySelector("#maxEntriesInput"),
  directoryText: document.querySelector("#directoryText"),
  modeText: document.querySelector("#modeText"),
  countText: document.querySelector("#countText"),
  filteredCountText: document.querySelector("#filteredCountText"),
  errorCountText: document.querySelector("#errorCountText"),
  refreshText: document.querySelector("#refreshText"),
  logList: document.querySelector("#logList"),
  detailMeta: document.querySelector("#detailMeta"),
  jsonTree: document.querySelector("#jsonTree"),
  rowTemplate: document.querySelector("#rowTemplate"),
};

ui.maxEntriesInput.value = String(DEFAULT_MAX_ENTRIES);
loadPersistedFilters(ui);
bindEvents();
renderAll();

function bindEvents() {
  ui.chooseDirBtn.addEventListener("click", chooseDirectory);
  ui.refreshBtn.addEventListener("click", () => scheduleLoad(false));
  ui.clearFiltersBtn.addEventListener("click", clearFilters);

  ui.autoRefreshToggle.addEventListener("change", () => {
    persistFilters(ui);
    configureRefreshMode(
      state,
      ui.autoRefreshToggle.checked,
      scheduleLoad,
      POLL_MS,
    );
    renderStats(ui, state);
  });

  ui.keywordInput.addEventListener("input", onFilterChanged);
  ui.levelSelect.addEventListener("change", onFilterChanged);
  ui.targetSelect.addEventListener("change", onFilterChanged);
  ui.startInput.addEventListener("change", onFilterChanged);
  ui.endInput.addEventListener("change", onFilterChanged);
  ui.maxEntriesInput.addEventListener("change", onMaxRowsChanged);
}

async function chooseDirectory() {
  if (!window.showDirectoryPicker) {
    alert("showDirectoryPicker is unavailable. Use a Chromium-based browser.");
    return;
  }

  try {
    const handle = await window.showDirectoryPicker({ mode: "read" });
    state.directoryHandle = handle;
    state.directoryName = handle.name;
    state.entries = [];
    state.filteredEntries = [];
    state.selectedId = "";
    state.fileStates.clear();
    state.parseErrors = 0;
    ui.refreshBtn.disabled = false;

    await scheduleLoad(true);
    configureRefreshMode(
      state,
      ui.autoRefreshToggle.checked,
      scheduleLoad,
      POLL_MS,
    );
    renderStats(ui, state);
  } catch (error) {
    if (error?.name !== "AbortError") {
      console.error(error);
      alert("Failed to select directory.");
    }
  }
}

function onFilterChanged() {
  persistFilters(ui);
  recalculateFiltered();
  renderAll();
}

function onMaxRowsChanged() {
  ui.maxEntriesInput.value = String(getMaxEntries(ui.maxEntriesInput.value));
  trimEntries();
  persistFilters(ui);
  recalculateFiltered();
  renderAll();
}

function clearFilters() {
  ui.keywordInput.value = "";
  ui.levelSelect.value = "ALL";
  ui.targetSelect.value = "ALL";
  ui.startInput.value = "";
  ui.endInput.value = "";
  persistFilters(ui);
  recalculateFiltered();
  renderAll();
}

async function scheduleLoad(reset) {
  if (!state.directoryHandle) {
    return;
  }

  if (state.loading) {
    state.queuedReset = state.queuedReset || reset;
    return;
  }

  state.loading = true;
  try {
    await ingestLogs(reset);
  } finally {
    state.loading = false;
    const queuedReset = state.queuedReset;
    state.queuedReset = false;
    if (queuedReset) {
      await scheduleLoad(true);
    }
  }
}

async function ingestLogs(reset) {
  if (reset) {
    state.entries = [];
    state.fileStates.clear();
  }

  const files = await listLogFiles(state.directoryHandle);
  for (const item of files) {
    const file = await item.handle.getFile();
    const fileState = state.fileStates.get(item.name) ?? {
      offset: 0,
      line: 0,
      remainder: "",
    };

    if (file.size < fileState.offset) {
      fileState.offset = 0;
      fileState.line = 0;
      fileState.remainder = "";
    }
    if (file.size === fileState.offset) {
      state.fileStates.set(item.name, fileState);
      continue;
    }

    const chunk = await file.slice(fileState.offset, file.size).text();
    fileState.offset = file.size;
    parseChunk(
      item.name,
      chunk,
      fileState,
      (entry) => state.entries.push(entry),
      () => {
        state.parseErrors += 1;
      },
    );
    state.fileStates.set(item.name, fileState);
  }

  trimEntries();
  updateFilterChoices();
  recalculateFiltered();
  state.lastRefreshAt = new Date();
  renderAll();
}

function trimEntries() {
  const maxEntries = getMaxEntries(ui.maxEntriesInput.value);
  if (state.entries.length > maxEntries) {
    state.entries = state.entries.slice(state.entries.length - maxEntries);
  }
}

function updateFilterChoices() {
  const levels = new Set();
  const targets = new Set();
  for (const entry of state.entries) {
    levels.add(entry.level);
    targets.add(entry.target);
  }
  refillSelect(ui.levelSelect, [...levels].sort(), ui.levelSelect.value);
  refillSelect(ui.targetSelect, [...targets].sort(), ui.targetSelect.value);
}

function recalculateFiltered() {
  state.filteredEntries = applyFilters(state.entries, {
    level: ui.levelSelect.value,
    target: ui.targetSelect.value,
    startMs: ui.startInput.value ? Date.parse(ui.startInput.value) : null,
    endMs: ui.endInput.value ? Date.parse(ui.endInput.value) : null,
    tokens: ui.keywordInput.value.toLowerCase().split(/\s+/).filter(Boolean),
  });

  if (!state.filteredEntries.some((entry) => entry.id === state.selectedId)) {
    state.selectedId = state.filteredEntries[0]?.id ?? "";
  }
}

function renderAll() {
  renderStats(ui, state);
  renderList(ui, state, ui.keywordInput.value, onSelectRow);
  renderDetail(ui, state, ui.keywordInput.value);
}

function onSelectRow(id) {
  state.selectedId = id;
  renderAll();
}
