export function hasTauriBridge(): boolean {
  return typeof window !== 'undefined' && Boolean(window.__TAURI__ ?? window.__TAURI_INTERNALS__)
}
