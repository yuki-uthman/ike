-- D31: the feeds the Worker builds from Odoo on its own schedule. One row per
-- feed, holding the whole snapshot as JSON text, so the route serves it as it
-- was stored. The row is written only after a fully successful build, so a
-- failed run leaves the previous snapshot in place. The first feed is
-- 'quotations' (the open quotations, issue #8).
CREATE TABLE IF NOT EXISTS feed (
  name         TEXT PRIMARY KEY,
  json         TEXT NOT NULL,
  generated_at TEXT NOT NULL
);
