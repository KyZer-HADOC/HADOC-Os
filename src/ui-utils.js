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
