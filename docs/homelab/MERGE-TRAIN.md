# Merge train → homelab-0 (2026-10-01)

Each PR head was gated before merging: the four gates (`cargo test --workspace`, `clippy -D warnings`,
`fmt --check`, `build --locked`). Heads that predate `scripts/dev-check.sh` (#18, #19, #20) ran the same four
commands directly. After each merge, `main`'s tree was checked to be identical to the gated head (`git diff --quiet`).
Merge method: merge commit. No branch deleted.

## Aparatus

| Order | PR | Head (gated) | Gate | Tests | `main` after merge |
|---|---|---|---|---|---|
| — | — | — | — | — | `cdd3ef1` (start) |
| 1 | #18 Envelope Stage 0 | `afccfca` | direct | 71/0 | `6f226bd13c048807843022efdecd222848f85cff` |
| 2 | #19 Envelope Stage 1 | `c83a382` | direct | 81/0 | `75d074ece4411b25c7f96273dcb44c7c3188d253` |
| 3 | #20 Stage 1 validation | `7c41d68` | direct | 81/0 | `727b56fc6a9f86bf0ef38ce53db00af2e72da090` |
| 4 | #22 Local replica status on the chain | `9a26a5d` | dev-check.sh | 87/0 | `5a124af8f8ccecceee473f519d41798a10edce2c` |
| 5 | #30 Quarantine checker | `3e94b82` | dev-check.sh | 95/0 | **`2e359581cfff99e23874345d2bdba2474771d175`** |

#19, #20, #22 and #30 were retargeted from their stacked base to `main` before merging.
`scripts/dev-check.sh` on a checkout of the final `main`: all four gates passed, 95/0.

### Tag `homelab-0`: not pushed (blocked)

`git push origin homelab-0` was refused with **HTTP 403** by the session's git proxy (organisation egress policy:
tag pushes are not allowed from this session). It was not retried or routed around, and no branch was created in its
place. The annotated tag exists only in the session clone. To create it:

```bash
git fetch origin && git tag -a homelab-0 2e359581cfff99e23874345d2bdba2474771d175 -m "homelab-0: merge train #18 #19 #20 #22 #30" && git push origin homelab-0
```

### Second landing: `claude/homelab` (Aparatus #31)

| PR | Head (gated) | Gate | Tests | `main` after merge |
|---|---|---|---|---|
| #32 Two keys only (`ic-up.sh` stops whitelisting `"*"`; `rws ic key`) | `83aa1eb` | dev-check.sh | 96/0 | **`92e41f80efbace70fc348ea03750d59dddd80a68`** |

The tree of `main` is identical to the gated head. `scripts/dev-check.sh` on `main` passed all four gates, 96/0.

### Tag `homelab-0`: moved to `92e41f8`, still not pushed (403 again)

- **Why it moved:** `homelab-0` is what the anker installs, so it must not whitelist `"*"`. The tag now points to
  `92e41f8`, the first `main` where only the dev and agent principals can create or upgrade. Its earlier target was
  `2e35958`. That tag had never reached the remote.
- **The push:** the first push dropped with a sideband disconnect. A traced second push showed
  `HTTP/1.1 403 Forbidden` on `git-receive-pack`. That is the same block as before, so there was no further retry.

To create the tag:

```bash
git fetch origin && git tag -a homelab-0 92e41f80efbace70fc348ea03750d59dddd80a68 -m "homelab-0: merge train #18 #19 #20 #22 #30 plus #32 (two keys only)" && git push origin homelab-0
```

### Third landing (2026-10-01): PR #33, docs only

| PR | Head (gated) | Gate | Tests | `main` after merge |
|---|---|---|---|---|
| #33 Record PR #32 landing and the tag command | `6f7bb07` | dev-check.sh | 96/0 | **`5a19248dd022d7511628c919a783d5b432f620a7`** |

- The `main` tree is identical to the gated head.
- `scripts/dev-check.sh` on `main` passed all four gates: 96/0.
- `ic-up.sh` still whitelists only `$DEV_PRINCIPAL` and `$AGENT_PRINCIPAL`.

**Tag `homelab-0`: still not on the remote.**
- It stays at `92e41f8`, because #33 changed docs only.
- I made one traced push. I filtered its output too early, so I did not see the final HTTP status.
- I ran a second plain push to read the outcome. It ended `unexpected disconnect while reading sideband packet` / `the remote end hung up unexpectedly`, the same failure as the earlier 403.
- `git ls-remote origin refs/tags/homelab-0` returns nothing.
- No further retries. The command to run is unchanged (see above).

### Fourth landing (2026-10-01): PR #34, docs only

| PR | Head (gated) | Gate | Tests | `main` after merge |
|---|---|---|---|---|
| #34 Record PR #33 and NAP #35 | `572a4a6` | dev-check.sh | 96/0 | **`6f80ae0534ca339d3ddf99de217e4938c72836aa`** |

The tag was not pushed this time, as instructed. It now waits on Dex in **#35**: `homelab-0` at `5a19248` or a newer main.

### Issue pass (2026-10-01)

| Closed | Opened | Comment only |
|---|---|---|
| Aparatus #23 and #31 (both split; the tag went to #35) | Aparatus #35 (tag) | NAP #18 (left open, needs a second node) |
| NAP #12 (split → #38), #13, #7, #8, #23, #21 (split → #39), #1 (split → #37) | NAP #37 (N=7 and state sync), #38 (ingest → event_envelope), #39 (money states) | NAP #14, #15, #9, #10, #4: "deferred, not this session", still open |

### Later landings (2026-10-01, same session): code plus docs, each gated

| PR | Gate | Tests | `main` after merge |
|---|---|---|---|
| #36 `apparatus mcp serve` (stdio MCP bridge, mcp-gate client), M6.md, CANISTERS.md | dev-check.sh | 100/0 | `887232f4c713831e9c67681f5478da1350ef08c3` |
| #37 `rws quarantine --osv` (live OSV / RustSec / GHSA → Advice) | dev-check.sh | 102/0 | `2aa8d6525f5d3788bcb5cb71754e5b4586cbb7b5` |
| this PR: SESSION-METRICS.md and this table | dev-check.sh | 102/0 | (see the PR merge) |

## NAP-corpus

| Order | PR | Head | `main` after merge |
|---|---|---|---|
| — | — | — | `d6a0126` (start; holds the three research files) |
| 1 | #31 Land the local IC run (`claude/ic-local-run`: LOCAL-RUN.md, POC-NOTES.md, HOOKS.md) | `c397c39` | `9c8be7862eca5d14dc75bcb3570510b83184ec60` |
| 2 | #30 Canister spike (retargeted to `main`) | `2c9ce93` | **`fb3a312cd84f79fb2446247f5d5dfec260330438`** |

| 3 | #34 Identities, gate v2 behind a passed vote, INSTALL-ORDER, ledger mirror and identity register sketches | `84b30fb` | **`93c90e57eac84eb74828e2a8607fd9c0b6c16174`** |

| 4 | #35 SNS-shaped policy book v4 and PUBLIC-CANISTERS.md | `36536e7` | **`18bd5017b96412473c3bfde04183e2e84ad98b87`** |

After #35, `SNS-SHAPE.md` on `main` names the book-controlled notary hash
`2001a55538db6c8b39812bb01469bd1bd1ca94fa45931ea93d64c06ff81855a4`. That is the hash rebuilt from the head with moc 1.6.0
and the one installed in the run.

| 5 | #36 Daily-root canister (Group 6) and checkup sketch | `2b5c0ae` | **`25b5c37b32c276e28e29b021126ac6b12e149bf7`** |

Hashes rebuilt from the merged `main` match what is installed:

- daily root: `71a24604fa46b176616d8fad19e8f81b63b91a953be2ae84650fd9c2983b18e6`
- notary: `2001a55538db6c8b39812bb01469bd1bd1ca94fa45931ea93d64c06ff81855a4`

| 6 | #40 Lifeline and policy book v5 | `dc5e39f` | `e6890ec4f118e240c9f27f494ffad41f884a6f37` |
| 7 | #41 INSTALL-ORDER.md, the order that runs | `b4a69dd` | `e77fcb5e51e3e891aeaa309825bc7afb7bd55a5b` |
| 8 | #42 Ledger mirror and identity register canisters | `12dd55d` | `45dddc853a73b7fff21283dec8ee57a6d23e0c9c` |
| 9 | #43 Public groups 1–4 locally, group 6 reinstalled | `d1d8743` | `169b763bbd2004984cf71301078871dceba06fb5` |
| 10 | #44 mcp-gate canister | `b7d5ad4` | `800066537e217aa32e8ae12fd11cd7843c0c94e5` |
| 11 | #45 CORRECTIONS.md and the implemented-vs-analogy table | `587a730` | **`f7ea26112bba4c20a7020bae0856f81ca30482cb`** |

The three research files are unchanged. NAP-corpus has no Rust gates; its canister tests run against the replica (see DEV-PATH.md).
