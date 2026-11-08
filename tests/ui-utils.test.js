import test from "node:test";
import assert from "node:assert/strict";
import { escapeHtml, formatBytes } from "../src/ui-utils.js";

test("escapeHtml renders markup characters as plain text", () => {
  assert.equal(
    escapeHtml('<a data-x="1">O\'Reilly & friends</a>'),
    "&lt;a data-x=&quot;1&quot;&gt;O&#39;Reilly &amp; friends&lt;/a&gt;"
  );
});

test("formatBytes formats zero and binary units", () => {
  assert.equal(formatBytes(0), "0 B");
  assert.equal(formatBytes(1024), "1.0 KB");
});

test("formatBytes labels missing and invalid sizes", () => {
  assert.equal(formatBytes(null), "Unavailable");
  assert.equal(formatBytes(undefined), "Unavailable");
  assert.equal(formatBytes(-1), "Unavailable");
});


import { filterEntries, mergeDirectoryPage } from "../src/ui-utils.js";

test("mergeDirectoryPage appends only new filesystem paths and tracks offsets", () => {
  const first = mergeDirectoryPage([], {
    entries: [
      { path: "C:\\folder", name: "folder" },
      { path: "C:\\a.txt", name: "a.txt" }
    ],
    next_offset: 2,
    has_more: true
  });
  const second = mergeDirectoryPage(first.entries, {
    entries: [
      { path: "C:\\a.txt", name: "a.txt" },
      { path: "C:\\b.txt", name: "b.txt" }
    ],
    next_offset: 4,
    has_more: false
  });

  assert.deepEqual(second.entries.map(entry => entry.name), ["folder", "a.txt", "b.txt"]);
  assert.equal(second.nextOffset, 4);
  assert.equal(second.hasMore, false);
});

test("filterEntries searches names case-insensitively and preserves text literally", () => {
  const entries = [
    { name: "AppData" },
    { name: "Program Files" },
    { name: "<notes>.txt" }
  ];

  assert.deepEqual(filterEntries(entries, "app"), [entries[0]]);
  assert.deepEqual(filterEntries(entries, "<NOTES>"), [entries[2]]);
  assert.deepEqual(filterEntries(entries, ""), entries);
});


import { isLowResourceDevice, shouldRefreshDashboard } from "../src/ui-utils.js";

test("isLowResourceDevice selects low-effects mode for dual-core systems", () => {
  assert.equal(isLowResourceDevice({ logical_threads: 2, total_memory: 8 * 1024 ** 3 }), true);
});

test("isLowResourceDevice selects low-effects mode at or below 4 GiB RAM", () => {
  assert.equal(isLowResourceDevice({ logical_threads: 4, total_memory: 4 * 1024 ** 3 }), true);
});

test("isLowResourceDevice leaves a higher-spec system in balanced mode", () => {
  assert.equal(isLowResourceDevice({ logical_threads: 8, total_memory: 16 * 1024 ** 3 }), false);
});

test("shouldRefreshDashboard pauses monitoring while hidden or gaming", () => {
  assert.equal(shouldRefreshDashboard({ documentHidden: true, gamingMode: false }), false);
  assert.equal(shouldRefreshDashboard({ documentHidden: false, gamingMode: true }), false);
  assert.equal(shouldRefreshDashboard({ documentHidden: true, gamingMode: true }), false);
  assert.equal(shouldRefreshDashboard({ documentHidden: false, gamingMode: false }), true);
});

import test from "node:test";
import assert from "node:assert/strict";
import { gameProfileLabel, formatBenchmarkMetric } from "../src/ui-utils.js";

test("gameProfileLabel names balanced and low-resource game presets", () => {
  assert.equal(gameProfileLabel("balanced"), "Balanced");
  assert.equal(gameProfileLabel("gaming"), "Gaming / Low Resource");
  assert.equal(gameProfileLabel("unknown"), "Balanced");
});

test("formatBenchmarkMetric returns a safe formatted metric", () => {
  assert.equal(formatBenchmarkMetric(12.345, "MB/s"), "12.3 MB/s");
  assert.equal(formatBenchmarkMetric(0, "M/s"), "0.0 M/s");
  assert.equal(formatBenchmarkMetric(null, "MB/s"), "Unavailable");
  assert.equal(formatBenchmarkMetric(Number.NaN, "MB/s"), "Unavailable");
});
