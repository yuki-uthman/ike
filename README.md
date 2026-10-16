# Ike Sales

A small, free dashboard of daily takings at MRH Investment — the money that
actually arrived each day, counted once each — with a follow-up list of open
quotations and a way for staff to record expenses. The page itself is **static**:
one `index.html`, no secret, no build step. It reads its data live from
[ike-data](https://github.com/yuki-uthman/ike-data), the shared Odoo pipeline.
The one exception is the Expenses pane, which saves through a small Cloudflare
Worker kept in this repo (see below).

## Three panes: Sales, Quotations and Expenses

The page is three panes side by side — **Sales** on the left (the default),
**Quotations** in the middle (the follow-up list that used to be its own repo,
`ike-quotations`) and **Expenses** on the right. Swipe left or right to move
between them, or tap a tab in the bar along the bottom. The bar's indicator follows the swipe as it
happens, so it always shows which pane you are on.

Two things follow from the swipe belonging to the pager:

- **Swiping no longer changes the day in Sales.** Pick a day from the pills or
  the bars, or use the left/right arrow keys on a keyboard.
- **A drag that starts on a chart or on the day pills scrolls that strip**, and
  stops there at its ends. Start the swipe anywhere else, or tap the tab.

Each pane keeps its own accent: teal for money banked, amber for money still
awaited, plum for money spent. The panes share one file but not their code —
every id and class in a pane is scoped to it (`s-…` / `q-…`,
`.pane-sales` / `.pane-quotes` / `.pane-expenses`).

The selected day is shared by all three panes: pick a day in one and the others
follow. (Quotations' **All** stays local to that pane.)

## One-time setup

**Enable GitHub Pages**: Settings → Pages → Source: "Deploy from a branch" →
Branch: `main`, folder `/ (root)`. The page will be at
`https://<your-username>.github.io/ike-sales/`.

For the Sales and Quotations panes that's it — there's nothing else to configure
here. Data freshness, the Odoo credential, and the de-duplication logic all live
in [ike-data](https://github.com/yuki-uthman/ike-data). The Expenses pane also
needs the Worker deployed (see below).

## Installing it on Android

The page is an installable web app. Open the Pages URL in Chrome on the phone,
tap the ⋮ menu, then **Install app** (or **Add to Home screen**). It gets the
Ike icon and opens full-screen with no browser bar.

- `manifest.webmanifest` names the app and lists the icons in `icons/`. The
  `maskable-*` icons are full-bleed so Android can cut them to any shape; the
  `icon-*` ones keep the rounded tile.
- `sw.js` exists to make the app installable and to open the shell with no
  signal. It never caches data: JSON and Expenses calls go straight to the
  network, and the page itself is fetched network-first.
- To change the icon, replace the files in `icons/` (192 and 512 px, plain and
  maskable) and keep the paths in the manifest.

## What a day's number means

**Money received that day**, not sales made that day — the same question
[ike-pos](https://github.com/yuki-uthman/ike-pos) (formerly ike-today) asks,
so the two dashboards can never disagree. A credit sale confirmed today does
not appear here until the payment lands; an old invoice settled today appears
here today.

The product list underneath is the products on whatever was paid for that
day. One caveat: a partial payment contributes its invoice's whole line set,
so in that case the products total more than the day's figure. MRH settles
invoices in full as a rule, so this is a stated edge rather than a routine
distortion.

A cheque, a card, or any other method that is neither cash nor transfer is
still money received, so it counts in this dashboard's total. ike-pos, which
has only a cash card and a transfer card to put it in, holds it out of both
and names it in a separate strip instead.

**This changed on 2026-09-16.** The dashboard used to count sales *made* each
day, paid or not. The full 60-day history was re-backfilled under the new
meaning at the same time, so the chart is comparable end to end.

## How it gets its data

`index.html` fetches
`https://raw.githubusercontent.com/yuki-uthman/ike-data/main/data/sales.json`
directly in the browser on every page load (`cache: 'no-store'`) — GitHub
serves raw file content with `Access-Control-Allow-Origin: *`, so this works
cross-repo with no server or API needed. See
[ike-data](https://github.com/yuki-uthman/ike-data)'s README for how that
file gets refreshed and what "Point of Sale" vs "Sales Orders" actually
counts.

## The Quotations pane

Every quotation and sales order that still has money owing on it — what hasn't
closed yet, so it can be chased. Two cards: **Quotations** that were never
confirmed, and **Unpaid orders** that were. Above them, a bar per day showing
how much still-open value was raised that day. It opens on today; **All**, the
pinned pill on the left, is the whole pipeline.

**Still open** means any part of the value is unsettled, and a record leaves the
moment the last rufiyaa arrives — by invoice or at the POS. "Paid" is
*computed*, not read: `invoice_status` and `invoice_ids` both lie on this Odoo
instance, so the number behind each row is
`amount_total − (paid invoice value + POS-settled value)`. A part-paid order
shows its remaining balance, with the total beneath. The reasoning lives in
`ike-data/scripts/fetch_quotations.py`.

**Not interested ×** (inside an expanded row) hides a dead lead and takes its
value out of the totals and the day's bar. It does **not** touch Odoo, and it
is remembered only in that browser's `localStorage` — dismiss on a laptop and it
is still there on a phone. A counter at the bottom of the pane lists what is
hidden and restores it.

It reads
`https://raw.githubusercontent.com/yuki-uthman/ike-data/main/data/quotations.json`
on load, then again every 60 seconds and whenever the tab becomes visible
(the file itself refreshes every 15 minutes). Open rows are remembered across a
refresh. Unlike `sales.json` it keeps no history — it is a live snapshot, and
its day span shrinks as old quotations settle. The pane shows **customer names,
order references and amounts owing**, so the visibility note below applies to
it in full.

## The Expenses pane

Staff record a **bank-transfer expense** from a phone: pick a category, enter
the amount in MVR, optionally type a description and attach a receipt photo,
then enter the shared PIN and save. The day's entries are listed under day pills
(oldest on the left, today on the right), each with a status chip.

What it does and does not do, by design:

- **It only creates a draft `hr.expense` in Odoo.** It never submits, approves,
  posts or pays. The accountant does that in Odoo.
- **Save first, send later.** The entry is stored in the Worker's own database
  immediately, then synced to Odoo. A failed send is retried on a schedule and
  can be retried by hand; an entry is never lost and never duplicated in Odoo.
- **Status chips** say where an entry stands: *Waiting to send*, *Not sent*
  (with the next retry time and a Retry button), *Draft* (in Odoo, not yet
  confirmed) and the confirmed states read back from Odoo.
- **The PIN is one shared key.** It authorises saving; it does not identify a
  person. The owner changes it in the Cloudflare dashboard, and wrong guesses
  are rate-limited.
- **The expense date is the Maldives date it was saved**, not the date it
  reached Odoo.
- The description Odoo shows is exactly what staff typed, or the category when
  they typed nothing.

The confirmed and pending spend shown here is read from
`https://raw.githubusercontent.com/yuki-uthman/ike-data/main/data/expenses.json`,
like the other panes. Only saving, listing the day's entries, the category list
and retry go to the Worker.

### The Worker

`worker/` is a Cloudflare Worker (name `odoo`) with a D1 database (`ike-odoo`).
It is the only part of the three ike repos that writes to Odoo, and it holds the
Odoo key as a Worker secret; the page never sees it. It checks the PIN and the
page's origin, and a Cloudflare cron every 15 minutes retries unsent entries and
reads their status back from Odoo (that read is read-only).

The decisions behind all of this are in `docs/product/brief.md` and
`docs/product/architecture/brief.md`; `worker/migrations/` is the database
schema. `npm test` runs the acceptance tests in `tests/` against the page in a
real browser and the Worker under Miniflare, with no call to the real Odoo.
Deploying the Worker is a manual step by the owner.

## Visibility

GitHub Pages on a free plan requires a **public** repository, so this page
is technically reachable by anyone with the exact URL, though it isn't
linked or indexed anywhere. If that's ever a concern, GitHub Pages on
private repos requires GitHub Pro or higher.
