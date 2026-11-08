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
