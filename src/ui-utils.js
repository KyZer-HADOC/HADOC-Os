const HTML_ESCAPES = {
  "&": "&amp;",
  "<": "&lt;",
  ">": "&gt;",
  '"': "&quot;",
  "'": "&#39;"
};

export function escapeHtml(value) {
  return String(value ?? "").replace(/[&<>"']/g, character => HTML_ESCAPES[character]);
}

export function formatBytes(value) {
  if (!Number.isFinite(value) || value < 0) return "Unavailable";
  if (value === 0) return "0 B";

  const units = ["B", "KB", "MB", "GB", "TB", "PB"];
  const unitIndex = Math.min(Math.floor(Math.log(value) / Math.log(1024)), units.length - 1);
  const amount = value / (1024 ** unitIndex);
  return `${amount.toFixed(unitIndex === 0 ? 0 : 1)} ${units[unitIndex]}`;
}


export function mergeDirectoryPage(currentEntries, page) {
  const entries = [...currentEntries];
  const knownPaths = new Set(
    currentEntries.map(entry => entry.path).filter(path => typeof path === "string")
  );

  for (const entry of page?.entries ?? []) {
    if (!entry || typeof entry.path !== "string" || knownPaths.has(entry.path)) continue;
    entries.push(entry);
    knownPaths.add(entry.path);
  }

  const rawOffset = Number(page?.next_offset);
  return {
    entries,
    nextOffset: Number.isFinite(rawOffset) ? Math.max(0, Math.floor(rawOffset)) : entries.length,
    hasMore: Boolean(page?.has_more)
  };
}

export function filterEntries(entries, query) {
  const needle = String(query ?? "").trim().toLocaleLowerCase();
  if (!needle) return entries;

  return entries.filter(entry =>
    String(entry?.name ?? "").toLocaleLowerCase().includes(needle)
  );
}


export function isLowResourceDevice(snapshot) {
  const logicalThreads = Number(snapshot?.logical_threads);
  const totalMemory = Number(snapshot?.total_memory);
  const hasLowThreadCount = Number.isFinite(logicalThreads) && logicalThreads > 0 && logicalThreads <= 2;
  const hasLowMemory = Number.isFinite(totalMemory) && totalMemory > 0 && totalMemory <= 4 * 1024 ** 3;
  return hasLowThreadCount || hasLowMemory;
}

export function shouldRefreshDashboard({ documentHidden = false, gamingMode = false } = {}) {
  return !documentHidden && !gamingMode;
}
