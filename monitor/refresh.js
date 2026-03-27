export function configureRefreshMode(
  state,
  autoRefreshEnabled,
  onLoad,
  pollMs,
) {
  stopRefresh(state);

  if (!state.directoryHandle) {
    state.refreshMode = "idle";
    return;
  }

  if (!autoRefreshEnabled) {
    state.refreshMode = "manual";
    return;
  }

  if (startFileSystemObserver(state, onLoad)) {
    return;
  }

  state.pollingId = window.setInterval(() => {
    onLoad(false);
  }, pollMs);
  state.refreshMode = `polling(${pollMs}ms)`;
}

export function stopRefresh(state) {
  if (state.pollingId !== null) {
    window.clearInterval(state.pollingId);
    state.pollingId = null;
  }

  if (!state.observer) {
    return;
  }

  if (typeof state.observer.disconnect === "function") {
    state.observer.disconnect();
  } else if (typeof state.observer.unobserve === "function") {
    try {
      state.observer.unobserve(state.directoryHandle);
    } catch {
      // noop
    }
  }

  state.observer = null;
}

function startFileSystemObserver(state, onLoad) {
  if (!("FileSystemObserver" in window)) {
    return false;
  }

  try {
    const observer = new window.FileSystemObserver(() => {
      onLoad(false);
    });

    if (typeof observer.observe !== "function") {
      return false;
    }

    observer.observe(state.directoryHandle, { recursive: false });
    state.observer = observer;
    state.refreshMode = "observer";
    return true;
  } catch (error) {
    console.warn("FileSystemObserver fallback to polling", error);
    return false;
  }
}
