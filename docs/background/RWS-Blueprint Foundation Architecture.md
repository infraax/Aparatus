# RWS-Blueprint — Foundation Architecture v1.0
### The digital Rijkswaterstaat: a personal data-infrastructure network on the Apple free tier
### 2026-07-17 · grounded in: PARKING Plan (philosophy), 4× platform research (constraints), VRCM-AMG v2 direction (first tenant)

> *"Wie de toekomst niet inbouwt in het heden, moet de toekomst twee keer betalen."*
> Whoever doesn't build the future into the present pays for the future twice. — This document exists so we pay once.

---

## 0 · Leeswijzer — how the philosophy maps to the machine

Every structural decision below is a direct translation of a Rijkswaterstaat principle. This table is the key; the rest of the document is the lock.

| Rijkswaterstaat principle | This system's embodiment |
|---|---|
| **Asset Owner / Asset Manager / Service Provider separation** | Dex (charter & risk tolerance) / the Core on the homelab (schemas, registries, canonical log) / the apps & modules (iOS app, widgets, Mac app, future lil Dex interface, Vector console) |
| **Omgevingswet — 26 laws → 1 framework law** | One event envelope schema governs every module, every surface, every future project. No module ever invents its own data law. |
| **Deltafonds — wettelijk verankerd, buiten politiek bereik** | The append-only JSONL canonical log. Structurally guaranteed: no app code can ever mutate or delete it, only append. The fund no feature can raid. |
| **MIRT trechtering — Verkenning → Voorkeursbeslissing → Projectbeslissing → Realisatie** | Phased roadmap (§9) with explicit go/no-go beslismomenten and evidence criteria per gate. |
| **Adaptief plannen + reserve capacity — "leg de buizen nu voor projecten over 5–10 jaar"** | Pre-laid hooks (§8): source types, mode stamps, module namespaces, provenance chains — fields that cost bytes today and prevent rewrites in 2028. |
| **Nachtonderhoud — night = 5h window, weekend = 50h, users barely notice** | The always-on MacBook is the maintenance depot: automated re-signing, syncs, projections, backups — all in defined windows, invisible to the user (§7). |
| **BIM clash detection — conflicts found before the shovel hits the ground** | Registry + schema validation in CI. A module whose events violate the envelope, collide with an existing namespace, or skip provenance fails validation before it ever writes to the canon. |
| **20.000 meetpunten — the road is also an information system** | Passive telemetry woven into every interaction: response latency, notification→answer lag, app-open events, dismissals. The road measures itself. |
| **Lifecycle cost management — total cost over the asset's life, never lowest build price** | Every platform/stack decision below is rated on 10-year cost, including the $99/year question, which becomes a data-driven beslismoment, not an ideology (§10). |
| **Transparantie over falen** | The system logs its own failures as first-class events: missed re-signs, sync gaps, notification droughts. Rotterdam-style honesty: when hinder is unavoidable, say so in the record. |
| **Scheiding beleid / uitvoering** | The charter (policy) lives in a versioned document the code reads but never edits. Execution lives in code. Neither reaches into the other. |

---

## 1 · The Authority Model — three layers, hard boundaries

```
┌──────────────────────────────────────────────────────────────┐
│  ASSET OWNER — Dex                                           │
│  Owns: the Charter (invariants, risk tolerance, privacy      │
│  rules, friction gate), version-bump authority, module       │
│  admission decisions. Touches no runtime code.               │
└──────────────────────────┬───────────────────────────────────┘
                           │ charter.yaml (read-only to all code)
┌──────────────────────────▼───────────────────────────────────┐
│  ASSET MANAGER — "the Core" (homelab, always-on MacBook)     │
│  Owns: canonical event log (append-only JSONL),              │
│  registries (constructs, items, modules, flags, event        │
│  types), the sync API, projection engine (scores, MDC,       │
│  EEI, trends), schema validation, backups, maintenance       │
│  scheduling. The hoofdwegennet.                              │
└──────┬───────────┬───────────┬───────────┬───────────────────┘
       │ one API   │ one API   │ one API   │ one API — the only road
┌──────▼─────┐ ┌───▼─────┐ ┌───▼──────┐ ┌──▼──────────────────┐
│ SERVICE    │ │ SERVICE │ │ SERVICE  │ │ SERVICE PROVIDERS   │
│ PROVIDER   │ │ PROVIDER│ │ PROVIDER │ │ (future)            │
│ iOS app    │ │ Widget  │ │ macOS    │ │ lil Dex corpus      │
│ (micro-    │ │ surface │ │ app      │ │ interface · Vector  │
│ prompts)   │ │ (lane   │ │ (heavy   │ │ console · anything  │
│            │ │ A/B/C)  │ │ surfaces)│ │ that earns a permit │
└────────────┘ └─────────┘ └──────────┘ └─────────────────────┘
```

**The three boundary rules (non-negotiable, charter-level):**

1. **Service Providers speak only envelope-events to the Core API.** No module reads another module's data directly; no module opens the JSONL file itself. All traffic over the hoofdwegennet, none over sluiproutes.
2. **The Core never contains module logic.** It stores, validates, projects, serves. The moment VRCM-AMG scoring rules live *inside* the sync server rather than in a registered projection, the separation is dead.
3. **The Charter is read by code, written only by the Owner.** Policy and execution never merge.

**Why this is the load-bearing design:** adding lil Dex's corpus interface in 2027 or a Vector telemetry module in 2028 is then a *permit application* — new module ID, new event-type namespace, registry entry, schema validation pass — and zero lines changed in the Core or any existing module. That is the Amsterdam move: the pipes for the 2028 project are laid in this document.

---

## 2 · Layer 0 — The Canon (the Deltafonds)

**Location:** always-on MacBook (homelab), Tailscale-reachable. Later: MAP infrastructure for the twin corpus, migrated by copying files — because the canon is files.

**Form:**
- `events/YYYY/MM.jsonl` — append-only, one event per line, UTF-8, monthly rotation. The single source of truth. Passes the "opens in 2036 with no software" test.
- `registries/` — versioned YAML: `constructs.yaml`, `items.yaml`, `modules.yaml`, `event_types.yaml`, `flags.yaml`. IDs immutable, never reused; wording versioned.
- `charter.yaml` — the invariants (below), version-controlled.
- Everything under git. The homelab holds the repo; backups are `git push` + periodic encrypted snapshots (the YubiKey/vault design comes later — the *hook* for it is that the canon is already a directory, not a database).

**The event envelope (the Omgevingswet — one law):**

```json
{
  "event_id": "uuid-v7",
  "captured_at": "2026-07-17T09:41:03Z",
  "local_tz": "Europe/Amsterdam",
  "event_time": null,
  "window": "today | past_4_weeks | past_12_months | none",
  "source": "self_report | passive | derived | external_ingest | system",
  "module": "vrcm_amg | core | corpus | vector | ...",
  "event_type": "vrcm_amg.item_response | core.resign_completed | ...",
  "instrument_version": "vrcm-amg/2.0",
  "item_id": "B1.4#v2 | null",
  "cadence_tier": "baseline | quarterly | monthly | daily | backtest | none",
  "mode": "observe | adaptive",
  "device": "iphone | macbook_daily | macbook_homelab",
  "channel": "app | widget | notification_action | shortcut | api",
  "payload": { },
  "provenance": ["stated" , "..."],
  "schema_version": "1.0"
}
```

**Charter invariants (the constitution of the canon):**
1. Append-only. No mutation, no deletion. Corrections are new events referencing the corrected `event_id`.
2. Every event carries full envelope. Validation rejects partials — no exceptions for "quick" writes.
3. Raw is canonical; every projection is recomputable and disposable.
4. Provenance is mandatory: the corpus must always distinguish *Dex said* from *system measured* from *system inferred*.
5. Gaps are data: dismissals, snoozes, silence-days are events, not absence.
6. `mode: observe` for all of year one; `adaptive` events are version-stamped and never silently mixed.
7. Failures are events (`core.resign_failed`, `core.sync_gap`) — transparantie over falen is a schema requirement, not a virtue.
8. Sensitive-scope rule: the canon stores instrument data and telemetry; it does not become a diary by accident. New data categories require a charter amendment (Owner decision), not a code push.

---

## 3 · Layer 1 — The Core (Asset Manager)

Runs on the always-on MacBook. Four small components, deliberately boring:

- **Sync API** — minimal HTTPS service over Tailscale (recommendation: Python FastAPI or Swift Vapor; my pick is **FastAPI** — the Claude Code build lane, your stated JSON/Python schema plan, and the analysis ecosystem all live in Python; the Mac side stays a systemd-style launchd service either way). Endpoints: `POST /events` (validate → append), `GET /state` (projections for widgets/apps), `GET /schedule` (what prompts are due), `GET /registry/*`.
- **Validator** — the BIM layer. JSON-Schema for the envelope + registry cross-checks (item exists, module registered, event_type in namespace, version known). Runs on every write *and* in CI on every registry/schema change. Clashes surface before the shovel, never after.
- **Projection engine** — pure functions over the log: axis scores, MDC corridors, EEI, flags, trends, the year-end Wrapped. Recomputable from zero at any time; projections are cached views, never sources.
- **Maintenance scheduler** — launchd jobs owning the nacht-windows (§7).

**What the Core is not:** it is not a big framework, not a container fleet, not a message queue. RWS manages 3,102 km of highway with discipline, not with exotica. One process, one log, one API, one schedule.

---

## 4 · Layer 2 — Service Providers (the surfaces)

### Surface allocation — the most important platform decision in this document

The research is unanimous on one asymmetry: **a locally-built macOS app runs indefinitely with no expiry; the iOS free-tier app dies weekly.** Therefore:

> **Put the smallest possible surface on the most constrained platform.**

- **iPhone (free-tier SwiftUI app):** micro-interactions only. The daily 3–5, notification actions (Answer/Snooze/Leave-me-alone buttons — fully supported free-tier), the <10-second loop, latency capture. Nothing heavy lives here. If the app dies mid-week (re-sign failure), the *loss surface is minimal* — a day of dailies, which the gap-is-data rule records honestly.
- **macOS app (daily-driver MacBook, ad-hoc signed, immortal):** the heavy surfaces — baseline and quarterly sessions, dashboards, bullet-grid + glyph visualizations, trend views, corpus browser, registry editor. Free of the 7-day clock entirely.
- **Homelab MacBook:** headless Core + maintenance depot; optionally the same macOS app for admin.

This allocation converts the free tier's worst constraint from a system-level risk into a contained, monitored, low-stakes maintenance item.

### The widget question — three lanes, swap without touching the Core

The research disagrees internally on free-tier widget data-sharing (App Groups blocked; but extensions build, and interactive App-Intent widgets are claimed to work on Personal Team). We do not resolve this on paper — we resolve it at milestone M3 with a device in hand. The architecture makes the answer unimportant:

- **Lane A — native WidgetKit extension**, same build, data via direct shared-file/Keychain pattern (App-Groups-free). Best interaction if entitlement reality cooperates.
- **Lane B — native WidgetKit as thin client:** the widget fetches `GET /state` from the Core directly over Tailscale on timeline refresh. Architecturally the *purest* lane — widget and app are both just clients of the hoofdwegennet — at the cost of needing the tunnel up.
- **Lane C — Scriptable JS widget** (store-distributed host app, no signing clock, Lock-Screen capable, reads the same `GET /state`). The guaranteed-to-work fallback; not interactive, tap-to-open only.

All three consume the same API and emit the same events. The lane is a Service-Provider implementation detail — exactly why the layer separation exists.

### The module contract (permit system for everything future)

A new module — corpus interface, Vector console, anything — must file:
1. A `modules.yaml` entry: module ID, owner, purpose, event-type namespace (`vector.*`), data categories touched.
2. Schemas for its event payloads (validated in CI).
3. A friction-gate justification for any manual interaction it introduces (charter §: acquisition ladder — passive ≻ ambient ≻ prompted-micro ≻ prompted-session).
4. Nothing else. No Core changes. No cross-module reads. Admission is an Owner decision recorded as a `core.module_admitted` event.

---

## 5 · The Notification Engine (free-tier native, fully local)

- `UNCalendarNotificationTrigger` schedules the daily windows (morning / afternoon / evening slots per your segmentation), monthly and quarterly session reminders, and the annual baseline call-up.
- **The 64-pending cap** governs the design: a rolling scheduler maintains only the next ~7 days of notifications and re-tops the window on every app open + background refresh. Scheduling more than 64 can silently kill *all* of them — this is a validator rule, not a comment.
- **Notification actions** carry the <10s loop: Answer (inline into a lightweight sheet), Snooze (re-offer in N hours), and the charter-mandated **"Leave me alone today"** — which fires a `daily.optout` event. Dignity is an API surface.
- No remote push on free tier — and by design we don't need it in observe-mode: year one's schedule is fully local and pre-computable. The adaptive-mode future (server-triggered prompts) is exactly what the $99 beslismoment (§10) will price against real data. A PWA Web-Push side channel exists as a documented reserve pipe; not built now.

---

## 6 · The Passive Layer — 20.000 meetpunten, honestly scoped

**Settled by all three research reports and accepted as terrain, not fought:** Screen-Time-class usage export is architecturally impossible on every platform, every tier. The `DeviceActivityReport` sandbox has no exit. We do not design around a door that does not exist.

What the road *can* measure about itself, from day one, at zero interaction cost:
- Per-item response latency (research-validated as cognitively informative)
- Notification-delivered → response lag; snooze chains; dismissals
- App-open / session events, device + channel per event
- Answer-changed events (hesitation is signal)
- Re-sign, sync, and gap system events (the network's own health)

Reserve pipes (hooks laid, not built): HealthKit read (entitlement status on free tier is contested between reports — **verify on-device at M1**; designed as an optional ambient module either way, never load-bearing), and ambient homelab sources (git commit timestamps, calendar density) as `external_ingest` events — the passive A4-duration channel from the v2 research, entering through the same front door as everything else.

---

## 7 · Nachtonderhoud — the maintenance protocol

The always-on MacBook is the wegendistrict. Same Apple ID as the iPhone (recommendation, see §12), so it holds the Personal Team signing identity.

| Window | Cadence | Jobs |
|---|---|---|
| **Nacht (03:00–06:00 daily)** | daily | Projection recompute · JSONL integrity check (line-count + hash chain) · git commit + encrypted snapshot · sync-gap audit (yesterday's expected vs received events → `core.sync_gap` if short) |
| **Re-sign window (every 5 days, 03:00)** | 5-daily | `xcodebuild` → install to iPhone over Wi-Fi (`devicectl`) → verify launch → `core.resign_completed` event. Two-day margin before the 7-day cliff; one automatic retry next night; on second failure → alert to Dex (ntfy/email from the Mac) + `core.resign_failed` event |
| **Weekend (50h, monthly)** | monthly | Registry/schema CI over the full canon · restore-test one backup (a backup that's never restored is a hope, not a backup) · dependency updates in a branch |
| **Zomervakantie (annual)** | yearly | Form version work, schema migrations-as-projections, the big renovations — scheduled when traffic is structurally light, exactly like the A15 |

The weggebruiker merkt het nauwelijks: every job runs while Dex sleeps, every job logs an event, and hinder that can't be avoided (a re-sign failure day) is announced, Rotterdam-style, not hidden.

---

## 8 · Reserve Capacity — the pipes laid now for 2028

Explicitly pre-built into the schema and registries although nothing uses them yet — each is a one-field cost today against a rewrite cost later:

1. `module` + namespaced `event_type` — the corpus interface and Vector console slot in without core changes.
2. `source: external_ingest` — ambient/passive channels (commits, calendar, robot telemetry) already have a legal entry road.
3. `mode: observe|adaptive` — the sensor→steering transition is a stamp, not a migration.
4. `provenance` chain on every event — twin-corpus compatibility is not a later export project; the canon *is* corpus-formatted from event one.
5. `cadence_tier` on every item — the AuADHD1000 yearly architecture is pre-tagged from v2's first draft.
6. `channel` field — a future watch surface, voice entry via Shortcuts, or Vector-as-input-device are values, not versions.
7. Registry `deprecated:` flags instead of deletions — IDs are forever, like hectometerpaaltjes.
8. The API's `GET /state` shape is module-agnostic — any renderer (SwiftUI, Scriptable, a future web dashboard, lil Dex himself) reads the same projection contract.

---

## 9 · MIRT Roadmap — trechtering with beslismomenten

**Verkenning** ✅ — platform research (4 reports), brede aanpak complete.
**Voorkeursbeslissing** ✅ — this blueprint: free-tier native + homelab Core + surface allocation. Signed by the Asset Owner by proceeding.

**Projectbeslissingen** (each gate has evidence criteria; adaptief — build what's needed now, pipes for later):

| Gate | Delivers | Go/no-go evidence |
|---|---|---|
| **M0 · Fundament** | charter.yaml · envelope schema · registries skeleton · JSONL store + validator + `POST /events` on homelab | An event round-trips: written, validated, appended, re-read. CI rejects a malformed one. |
| **M1 · Eerste rijstrook** | iOS skeleton app: daily 3–5 UI, notification actions, local scheduler (64-window manager), sync queue, latency capture. HealthKit free-tier verification. | Seven consecutive days of dailies captured end-to-end, gaps included, visible in the canon. |
| **M2 · Nachtonderhoud** | Automated re-sign pipeline + all launchd windows live | **Three consecutive re-sign cycles fully automatic.** This gate *is* the $99 beslismoment: if automation holds, free tier stands; if it costs manual intervention, the fee is bought on lifecycle-cost evidence, not ideology. |
| **M3 · Widget lane** | Lanes A→B→C tested on-device in that order; one chosen | <10s notification/widget-to-answered loop measured, on the winning lane. |
| **M4 · Eerste tenant** | VRCM-AMG v2 module complete: item bank (cadence-tagged), scoring projections, flags, EEI v2 | v2 baseline run captured through the full pipeline; run-1 data re-imported as `external_ingest`. |
| **M5 · Zichtlaag** | macOS heavy app: bullet grid, glyph, MDC corridors, trend views | Dex reads his own state without opening a terminal. |
| **M6+** | Adaptive-mode design · corpus interface permit · Vector permit · Wrapped engine | Each a separate Projectbeslissing with its own verkenning. |

Parallel spoor: VRCM-AMG v2 *content* work (item bank, EEI redesign) continues independently — infrastructure and tenant are separate programmes sharing one MIRT-Overzicht.

---

## 10 · Lifecycle Cost Register

| Decision | Build cost | 10-year cost | Verdict |
|---|---|---|---|
| Free tier + automated re-sign | ~0 € | ~73 automated cycles/yr; risk = automation fragility + Apple policy drift | **Base case**; M2 gate converts this from belief to measurement |
| $99/yr Developer Program | 99 €/yr | ~990 € + zero re-sign risk + push/App Groups/TestFlight unlocked | Bought if and when M2 evidence or adaptive-mode needs justify; a config change, not an architecture change — *that* is the blueprint's real achievement |
| JSONL canon vs. database-as-truth | trivial | Zero lock-in, zero migration walls, greppable in 2036 | Locked, charter-level |
| FastAPI Core vs. Swift Vapor | comparable | Python: shares the Claude Code build lane + analysis stack; Vapor: one-language purity | **FastAPI recommended**; overrideable at M0 with zero downstream effect (the API contract is the boundary, not the language) |
| Native iOS vs. everything else | highest build effort | Only class satisfying micro-input + widgets + latency capture + local-first + owner-updates (all four reports concur) | Locked by Voorkeursbeslissing |

---

## 11 · Risk Register — transparantie over falen, pre-declared

| Risk | Likelihood | Blast radius | Mitigation |
|---|---|---|---|
| Re-sign automation breaks (Xcode update, Apple policy, Wi-Fi pairing loss) | Medium | Days of dailies on iPhone only; Mac + canon unaffected | Surface allocation caps the loss; alerting + manual fallback; M2 evidence gate; $99 escape hatch priced in |
| Apple tightens free-tier further | Low-Medium | Same as above | Same escape hatch; Scriptable lane C is signing-free |
| Widget data-sharing fails all free-tier patterns | Medium | Interaction quality, not data | Lane B/C fallbacks; notifications alone satisfy the <10s loop |
| HealthKit blocked on free tier | Contested in research | An ambient nice-to-have | Designed non-load-bearing; verify at M1 |
| Homelab MacBook dies | Low | Canon availability, not existence (git + snapshots) | Restore-test in the monthly window; canon is files → any machine resurrects it |
| Scope creep: foundation eats the instrument | **The real one** | Months | MIRT discipline: M0–M2 are deliberately thin; VRCM-AMG v2 content runs parallel; every "wouldn't it be cool" is a module permit application for later, filed in `modules.yaml` as `proposed` |

---

## 12 · Open decisions for the Asset Owner (recommendations included)

1. **Apple ID on the homelab MacBook: same ID.** One Personal Team = one signing chain = the depot can re-sign for the iPhone. Device limits (≈3/platform) fit your fleet. A separate ID would split signing authority from the maintenance depot — the one thing the depot exists to hold.
2. **Core stack: FastAPI** (per §10) — override at M0 costs nothing.
3. **Widget lane order: A → B → C** at M3, decided by device evidence.
4. **Charter ratification:** §2's eight invariants + §1's three boundary rules are the constitution. Amend now or they harden at M0.

---

*De cultuur volgt het systeem. The system is now on paper; M0 makes it real. One canon, one law, one road network, maintenance at night — and every future project finds its pipes already in the ground.*
