# Monitor Web MVP

A minimal local log monitor for Beluna core logs.

## Scope

This MVP reads local NDJSON log files from the browser using `showDirectoryPicker`, then provides:

1. Field filters (`level`, `target`, timestamp range).
2. Keyword search over JSON content.
3. Tree-style JSON detail explorer.
4. Auto refresh with `FileSystemObserver` when available, otherwise polling fallback.

## Run

`showDirectoryPicker` requires a secure context. Run the page via localhost:

```bash
cd monitor
python3 -m http.server 4173
```

Then open `http://localhost:4173`.

## Use

1. Click "Choose Log Directory".
2. Pick Beluna logs directory, usually `logs/core`.
3. Use filters and keyword search.
4. Click any row to inspect full JSON tree.

## Notes

- File name pattern consumed by default: `core.log.YYYY-MM-DD.N`
- Non-matching files are ignored.
- Broken JSON lines are skipped and counted in `Parse Errors`.
