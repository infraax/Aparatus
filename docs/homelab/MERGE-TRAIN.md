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

## NAP-corpus

| Order | PR | Head | `main` after merge |
|---|---|---|---|
| — | — | — | `d6a0126` (start; holds the three research files) |
| 1 | #31 Land the local IC run (`claude/ic-local-run`: LOCAL-RUN.md, POC-NOTES.md, HOOKS.md) | `c397c39` | `9c8be7862eca5d14dc75bcb3570510b83184ec60` |
| 2 | #30 Canister spike (retargeted to `main`) | `2c9ce93` | **`fb3a312cd84f79fb2446247f5d5dfec260330438`** |

| 3 | #34 Identities, gate v2 behind a passed vote, INSTALL-ORDER, ledger mirror and identity register sketches | `84b30fb` | **`93c90e57eac84eb74828e2a8607fd9c0b6c16174`** |

The three research files are unchanged. NAP-corpus has no Rust gates; its canister tests run against the replica (see DEV-PATH.md).
