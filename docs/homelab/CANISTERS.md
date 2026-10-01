# Canisters for the homelab (local replica)

All of these run on the local N=1 replica from `scripts/ic-up.sh`. They are not deployed to mainnet, and no cycles were spent.

- **Sources:** NAP-corpus `docs/ic/canisters/` (Motoko, mops, moc 1.6.0, `core` 2.6.2, `sha2` 0.2.5).
- **Installer:** NAP-corpus `docs/ic/canisters/tools/ic-install` (ic-agent 0.49.2). No dfx.
- **Order:** `docs/ic/canisters/INSTALL-ORDER.md` and `install-order.sh`.

| Canister | Version | `module_hash` | Controller | Doc (NAP-corpus `docs/ic/canisters/`) |
|---|---|---|---|---|
| lifeline | v1 | `f7a7b7603324516b0471c9f1084fcf6b9cc0a0b20ed4c845794631a7659153b0` | policy book | `SNS-SHAPE.md` |
| policy book | v5 | `2002151f77e8031180977cc9fd35607445278e7dec746ca868f0e1765be7e5d7` | lifeline | `SNS-SHAPE.md`, `POLICY-BOOK.md` |
| upgrade gate | v3 | `7d2b315fca8efed79a2b95350bfa873c4dc4ad26ad494cf6eda7089d3b30095c` | lifeline | `UPGRADE-GATE.md` |
| status notary | v5 | `2001a55538db6c8b39812bb01469bd1bd1ca94fa45931ea93d64c06ff81855a4` | policy book | `STATUS-NOTARY.md` |
| daily root (group 6) | v1 | `71a24604fa46b176616d8fad19e8f81b63b91a953be2ae84650fd9c2983b18e6` | policy book | `DAILY-ROOT.md` |
| ledger mirror | v1 | `eb20e091384c37a939933b69ceee87a9473cd30d660a9a0bad502286e62754a0` | policy book | `LEDGER-MIRROR.md` |
| identity register | v1 | `b0e543497929d2786ff4efd35462ccdc44a8036c25fae43aa1e70e291209ea75` | policy book | `IDENTITY-REGISTER.md` |
| mcp-gate | v1 | `9d5f4dc9947b250fe570427c7556278f6ac6e13f96d63c7e2210c56f51fb522d` | policy book | `MCP-GATE.md`; bridge: `apparatus mcp serve` (`docs/milestones/M6.md`) |
| heartbeat (group 1) | v1 | `4c913f10f84a6faacd6b6d5b5229c01d448deecdae9b4d09469f41ca294f0fe0` | policy book | `../PUBLIC-CANISTERS.md` |
| checkup (group 2) | v1 | `b80ae56709ddf2a45cde676684b026f093841727b0c760677e19a86156cbd51b` | policy book | `../PUBLIC-CANISTERS.md` |
| receipt (group 3) | v1 | `d0337238dca9135e052ebbd353ab8ce6c09294735beff483346295ff27eedefe` | policy book | `../PUBLIC-CANISTERS.md` |
| mesh summary (group 4) | v1 | `f00c754c6bf6090ba480a1b84948b918d75c6e2bfcfc3f6dfbdf254ef8dd8d2d` | policy book | `../PUBLIC-CANISTERS.md` |

**The dev key controls none of them.**

- It can propose and vote through its neuron.
- `install_code` from the dev key returns IC0512 everywhere.
- Every update on the daily root, the mirror, the public groups and mcp-gate must come from the anker key `.ic-local/anker.key`. Any other caller traps.

**Keys** live in `.ic-local/` with mode 0600, and that directory is git-ignored:

| Key | Role |
|---|---|
| `dev.key` | proposes, votes |
| `agent.key` | votes, records on the notary |
| `anker.key` | anker updates |
| `anker2.key` | the rotated anker on daily root `r7inp` |
| `third.key` | test |

The whitelist for canister creation covers only dev and agent.
