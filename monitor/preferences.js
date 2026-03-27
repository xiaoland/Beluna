export const DEFAULT_MAX_ENTRIES = 5000;

export function getMaxEntries(rawValue) {
  const parsed = Number.parseInt(rawValue, 10);
  if (Number.isNaN(parsed)) {
    return DEFAULT_MAX_ENTRIES;
  }
  return Math.max(200, Math.min(20000, parsed));
}

export function persistFilters(ui) {
  localStorage.setItem(
    "monitor.filters",
    JSON.stringify({
      keyword: ui.keywordInput.value,
      level: ui.levelSelect.value,
      target: ui.targetSelect.value,
      start: ui.startInput.value,
      end: ui.endInput.value,
      maxRows: getMaxEntries(ui.maxEntriesInput.value),
      autoRefresh: ui.autoRefreshToggle.checked,
    }),
  );
}

export function loadPersistedFilters(ui) {
  const raw = localStorage.getItem("monitor.filters");
  if (!raw) {
    return;
  }

  try {
    const data = JSON.parse(raw);
    ui.keywordInput.value = data.keyword ?? "";
    ui.startInput.value = data.start ?? "";
    ui.endInput.value = data.end ?? "";
    ui.maxEntriesInput.value = String(data.maxRows ?? DEFAULT_MAX_ENTRIES);
    if (typeof data.autoRefresh === "boolean") {
      ui.autoRefreshToggle.checked = data.autoRefresh;
    }
  } catch (error) {
    console.warn("Failed to load saved filters", error);
  }
}
