# NAP-corpus ↔ Aparatus compatibility plan

Dit bestand is de brug tussen de research in `infraax/NAP-corpus` en de runtime in `infraax/Aparatus`. Het is het enige bestand dat je nodig hebt om te weten: wat is klaar, wat komt, in welke volgorde, en wat hangt van wat af.

## 1. Wat is de research (NAP-corpus)

De repo bevat zes NAP-tracks (post-quantum crypto, ICP-subnet, netwerktopologie, LoRa/mesh, autonomous LLM hosting, corpus-schema), RWS-source-dossiers (Stage 0–2), Reticulum- en RF-sensing-research, dependency-management, en de RWS 2.0 blueprint. Alles is bronarchief: geen conclusies, alleen feiten met tags [A]/[B]/[C]/[NF].

Belangrijke docs:

- `docs/ic/ROADMAP.md` — de volgorde: PoC eerst, apparatus pas als laatste fase
- `docs/ic/BAREBONES-REPLICA.md` — Path B spec (stock replica + operator LocalStore)
- `docs/ic/CANISTER-MATRIX.md` — canister-selectie
- `docs/ic/MINI-GOVERNANCE.md` — proposal → vote → upgrade zonder tokens
- `docs/ic/VETKEYS-SPIKE.md` — vetKeys voor agent-identiteit
- `docs/ic/RWS-IC-MAPPING.md` — RWS-regels → IC-mechanismen

Issues: #12–#19 (envelope stages, IC-research, canister-selectie, PoC, RWS-mapping).

## 2. Wat is er al in Aparatus (M0–M3)

- **M1:** types, canonieke hash, schema/state, file store, CLI. 13 tests.
- **M2:** K-01 (keep vereist agreement), K-09 (displacement bij deferred), C-06 (correction rol-rang), V-01 (review subject), writer-lock, solo-init met 4 rollen. 48 tests.
- **M3:** `apparatusd` daemon met Unix socket, JSONL-RPC, één writer-actor, tickets als `report_back`-events, panic → ticket + herstart. 54 tests.
- **M4 (handoff):** ingest-pijp, signalen, backup/restore, hooks — nog niet gebouwd. Zie `docs/handoff/M4-HANDOFF.md`.

## 3. Envelope-ontwikkeling (de brug)

De envelop is de bouwsteen die research en runtime verbindt. Vier stages, elk een issue in NAP-corpus:

| Stage | Issue | Wat | Wanneer |
|---|---|---|---|
| **0** | #12 | Klassieke envelop-structuur: receipt-type, provenance-enum, lifecycle-veld, CID/BLAKE3-types, SigningKey-trait met Ed25519 + ML-DSA-stub | Nu — eerste upgrade |
| **1** | #13 | Sealing-afdwinging, CID-dedup, lifecycle-transities als schema-regels | Na Stage 0 |
| **2** | #14 | ML-DSA-65 implementatie, hybride modus (ed25519\|ml-dsa-65\|hybrid) | Na Stage 1 |
| **3** | #15 | Bitcoin-anchoring via threshold Schnorr — **langst uitgesteld** | Pas als 0–2 stabiel + barebones-subnet draait |

**Regel:** elke stage moet `cargo test --workspace` groen houden. De envelop is een laag op de bestaande ledger, geen vervanging.

## 4. Ontwikkelvolgorde en afhankelijkheden

```
M3 (daemon) ──► M4 (ingest/signalen/backup) ──► Envelope Stage 0
                                                      │
                                                      ▼
IC PoC (#18) ──► Canister-selectie (#17) ──► Envelope Stage 1
                                                      │
                                                      ▼
RWS-mapping (#19) ──► Mini-governance (#5 in NAP-corpus)
                                                      │
                                                      ▼
Envelope Stage 2 (ML-DSA) ──► Envelope Stage 3 (Bitcoin)
```

**Kernprincipe:** de IC-replica en de envelop zijn aparte machines. De replica is het anker (heartbeat, identiteitsregister, governance); de envelop is de structuur van elk object. Ze communiceren via de HTTP-endpoint van de replica, niet via de ledger zelf.

## 5. Wat NIET nu

- Bitcoin-anchoring (Stage 3) — wacht op stabiele envelop + werkende subnet
- ML-DSA-implementatie (Stage 2) — de stub volstaat; verificatie geeft bewust `true` terug
- Mesh-nodes als stemmers op operationele proposals — alleen op netwerk-proposals
- volledige NNS-nabouw — alleen het proposal-stem-upgrade-patroon

## 6. Voor de volgende sessie

1. Lees dit bestand + `NAP-corpus/docs/ic/ROADMAP.md`
2. Begin met Envelope Stage 0 (#12 in NAP-corpus)
3. Daarna M4 (ingest-pijp) of IC PoC (#18) — naar keuze, maar niet allebei tegelijk
4. Elke upgrade: check of een corpus-element erbij kan (hook, type, of module) zonder de runtime te breken

---

*Dit bestand is het compatibiliteitsplan. Het verandert alleen als de volgorde verandert — niet als de details veranderen.*