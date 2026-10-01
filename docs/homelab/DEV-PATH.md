# Dev path on homelab-0: one run from a fresh directory

Run 2026-10-01. Fresh clones in an empty directory (`$RUN`):

- Aparatus at `2e359581cfff99e23874345d2bdba2474771d175`. This is main after PR #30, and the commit the
  `homelab-0` tag points to. The tag exists only locally: the push returned 403, see `MERGE-TRAIN.md`.
- NAP-corpus at `fb3a312cd84f79fb2446247f5d5dfec260330438`, main after PR #30.
- Nothing came from the earlier working tree. The IC binaries were downloaded again and the chain was new.

## Result

| Step | Result |
|---|---|
| `scripts/ic-up.sh` | Downloaded the binaries, verified them against `SHA256SUMS`, and the replica came up. Exit 0. |
| `scripts/dev-up.sh` | Built with `--locked`, ran `rws init --solo`, and started apparatusd. Took 60 s. Exit 0. |
| Notary and policy book, installed with the dev identity | Both canisters were installed with the dev identity. Each Wasm `sha256` matched the published module hash bit for bit (notary v2 `727446b0…`, policy book v2 `be7ae122…`). |
| `rws ic status` writes a height | Height 573 went on the chain as a measured `event_envelope`. Exit 0. |
| Enact without a vote | Refused with exit 2, and `event 0 EnactRefused` was recorded in the policy book. |
| The refusal survives a replica restart | After `ic-down.sh` and `ic-up.sh`, `pb-show` returns the same event. The notary count is still 1. The next `rws ic status` read height 575. |
| `rws quarantine` scans `Cargo.lock` and does not rewrite it | 300 pins: 295 adopted, 5 waiting, 0 refused. Afterwards `cmp` reports the file unchanged, its sha256 is the same (`6ddf54f9…`), and `git status` is clean. |
| Chain | `rws check` → `check ok`, with 306 receipts. |

## What homelab-0 does not include yet

Parts 2 and 3 of this session live on `claude/homelab` (Aparatus and NAP-corpus). They are not on main:

- **Whitelist.** `ic-up.sh` at homelab-0 still bootstraps with `{"provisional_whitelist": ["*"]}`.
  - The dev key in this run was created by `ic-install` (`IC_INSTALL_KEY`, mode 0600) on its first call. That is why its principal (`ueuyw-…`) differs from the one in `IDENTITIES.md`.
  - On `claude/homelab`, `ic-up.sh` instead creates `dev.key` and `agent.key` and whitelists only those two principals.
- **Upgrade gate.** The notary here is controlled by the dev key directly. The gate (`upgrade-gate` v1, plus the controller handover) is on NAP-corpus `claude/homelab`, in `docs/ic/canisters/UPGRADE-GATE.md`.
- **Canister versions.** Main carries notary v1/v2 and policy book v1/v2. Notary v3/v4 and policy book v3 are on `claude/homelab`.

## Transcript

Paths shortened: `$RUN` = the fresh directory, `$APARATUS` = `$RUN/Aparatus`.
The quarantine output is cut to its head and tail; there is one receipt line per pin.

```text
# Aparatus 2e35958 (homelab-0 commit), NAP-corpus fb3a312; fresh clones in $RUN
$ cd Aparatus && scripts/ic-up.sh
downloading d26cd031176beec51b39fbb9e39e80a3a46a748e binaries
binaries verified
replica up (pid 6648): http://127.0.0.1:8080/api/v2/status
exit 0
$ scripts/dev-up.sh
initialised solo project in $APARATUS/.apparatus
apparatusd pid 12269, log $APARATUS/.apparatus/apparatusd.log
socket: $APARATUS/.apparatus/apparatusd.sock

real	0m59.845s
exit 0
$ cd NAP-corpus/docs/ic/canisters   # build notary v2 and policy book v2 with pinned moc (mops)
status-notary moc Motoko compiler 1.6.0 (source fk1pxi8x-6jsn2sh7-6cx3av0b-h2qknlvz)
status-notary build/v1.wasm sha256 a93570819999a63edeac96d7a4e280b58c3bdf74933c81d09d1f3b42ea36437e
status-notary build/v2.wasm sha256 727446b0d9c23668e4aa92a38ac9c8a06ba1aa163696c16928e2fe97349e5175
policy-book moc Motoko compiler 1.6.0 (source fk1pxi8x-6jsn2sh7-6cx3av0b-h2qknlvz)
policy-book build/v1.wasm sha256 b408a361c476b8440cb4a0d88fb2b3ca7ca2659a4ed02106df2da4c78c9ce0c1
policy-book build/v2.wasm sha256 be7ae122d75793ac084151eefd44f8a179b6f70c659eef1defcc59da81fa788e
$ (cd tools/ic-install && cargo build --locked -q)
exit 0
$ export IC_INSTALL_KEY=$APARATUS/.ic-local/dev.key; I=tools/ic-install/target/debug/ic-install
$ $I ident
principal ueuyw-rnm2i-5kjlt-2uoyo-k6g4l-4t4mr-zo6z6-l7mnz-cuzvm-nkvek-uqe
600 Aparatus/.ic-local/dev.key
$ $I create   # status notary
canister rwlgt-iiaaa-aaaaa-aaaaa-cai
$ $I install rwlgt-iiaaa-aaaaa-aaaaa-cai status-notary/build/v2.wasm
install ok, wasm sha256 727446b0d9c23668e4aa92a38ac9c8a06ba1aa163696c16928e2fe97349e5175
$ $I record rwlgt-iiaaa-aaaaa-aaaaa-cai 1 healthy
record -> count 1
$ $I module-hash rwlgt-iiaaa-aaaaa-aaaaa-cai
module_hash 727446b0d9c23668e4aa92a38ac9c8a06ba1aa163696c16928e2fe97349e5175
$ $I create   # policy book
canister rrkah-fqaaa-aaaaa-aaaaq-cai
$ $I install rrkah-fqaaa-aaaaa-aaaaq-cai policy-book/build/v2.wasm
install ok, wasm sha256 be7ae122d75793ac084151eefd44f8a179b6f70c659eef1defcc59da81fa788e
$ $I pb-advise rrkah-fqaaa-aaaaa-aaaaq-cai 'agent: raise keep envelope for asset dike-12 by 5%'
advice 0
$ $I pb-propose rrkah-fqaaa-aaaaa-aaaaq-cai 0 build
propose ok 0
$ $I pb-enact rrkah-fqaaa-aaaaa-aaaaq-cai 0   # no vote
enact refused: no passed vote: 0 yes of 0 voters, needs two thirds
exit 2
$ $I pb-show rrkah-fqaaa-aaaaa-aaaaq-cai
counts advices 1 proposals 1 votes 0 policies 0 events 1 voters 0
advice 0 by ueuyw-rnm2i-5kjlt-2uoyo-k6g4l-4t4mr-zo6z6-l7mnz-cuzvm-nkvek-uqe : agent: raise keep envelope for asset dike-12 by 5%
event 0 EnactRefused subject 0 by ueuyw-rnm2i-5kjlt-2uoyo-k6g4l-4t4mr-zo6z6-l7mnz-cuzvm-nkvek-uqe : no passed vote: 0 yes of 0 voters, needs two thirds
$ cd Aparatus && target/debug/apparatus rws ic status
ic status http://127.0.0.1:8080/api/v2/status healthy certified_height 573
event_envelope 01a0f820-9deb-722c-a291-1dbfdf24d613 cid bafkr4iftymhw3d75kdnhsydkxeq7s7nsfzol2ti4biaipg2pjwrhyu7fnq
01a0f820-9ded-702b-8c65-ee8a4609d501 event_envelope c01cdec14be3e1bc33c540b098be9a1cda068ba8587e415443f5259ae0eb5960
exit 0
$ scripts/ic-down.sh
replica stopped (pid 6648)
exit 0
$ scripts/ic-up.sh
binaries already verified against SHA256SUMS; no download
replica up (pid 17129): http://127.0.0.1:8080/api/v2/status
exit 0
$ $I pb-show rrkah-fqaaa-aaaaa-aaaaq-cai   # after the replica restart
counts advices 1 proposals 1 votes 0 policies 0 events 1 voters 0
advice 0 by ueuyw-rnm2i-5kjlt-2uoyo-k6g4l-4t4mr-zo6z6-l7mnz-cuzvm-nkvek-uqe : agent: raise keep envelope for asset dike-12 by 5%
event 0 EnactRefused subject 0 by ueuyw-rnm2i-5kjlt-2uoyo-k6g4l-4t4mr-zo6z6-l7mnz-cuzvm-nkvek-uqe : no passed vote: 0 yes of 0 voters, needs two thirds
$ $I count rwlgt-iiaaa-aaaaa-aaaaa-cai
count 1
$ target/debug/apparatus rws ic status
ic status http://127.0.0.1:8080/api/v2/status healthy certified_height 575
event_envelope 01a0f820-b713-7203-a806-9d724951cc75 cid bafkr4ig4clw4k4awkufxt4fpkqsbcof3qxsmjqwdymnwphbq3ydcphbzqa
01a0f820-b714-74b0-bd8f-c4a121242c2e event_envelope 4708c1c8b22c17c44590d5cd01fb0693ec2cface613512480beef31f426d2ab4
exit 0
$ cp Cargo.lock ../Cargo.lock.before && sha256sum Cargo.lock
6ddf54f90058d59d309787a1a92b7050d2a1f4708a49fb877fafe73b9d78ee4e  Cargo.lock
$ scripts/quarantine.sh
01a0f820-d901-74cc-875a-9b6ba4c4466d event_envelope fb46eff0312b97f7a688831e3aec7664faf03a91e7fd72ca0e3126189c866c7c
01a0f820-d90d-70d5-94fa-8a8497e66c73 event_envelope 1ad7fa3604132e0b8746558897564c9424ad1ee0ab622a4b148fd026574afcf9
01a0f820-d91a-75db-8817-c159cf506ba8 event_envelope 14357e92c6203b818d95cbc5e5f3500a1bb60c5b617718511e684e17877a722c
01a0f820-d926-76b8-aa44-3e5c3276d4b6 event_envelope e2154a186bc1a0eacc2f9837fb3c3b1284d25fb4a31de4b1f37ae7c9a3bbb0f9
01a0f820-d933-75f3-8688-664d1f3db342 event_envelope 29a9ae6737c8cf1267cf4af1bfcb872df87b679653712a1a38dcb75b545c23e8
... (306 lines)
01a0f820-e749-73f2-b559-f9fd7a48e5e8 event_envelope 62ed4d4030376247080ed54fcd1bc1db4b8a7afa61dcb07acfc04b9ba3ece709
01a0f820-e756-76ef-a82c-2ecb4cebd058 event_envelope 638a9b30b2ff4d419ee9b856871a198c328388802fc7f53b5de384345ea72181
01a0f820-e762-7118-bb46-ad50b8551a68 event_envelope a061411314da419ce1ba59a2dfff2f92c7b5540cd250b78ddd400f3f7be69293
01a0f820-e76f-7306-a1f0-2615ca9c7d54 event_envelope c9776c1c87a4d36324ae02c4cffb2ecc61571d0a4f2bd3f06e3fb18529317e4a
01a0f820-e77b-7271-b89f-cafa2947a864 event_envelope 5b4335aa1fc693315d2baa19e96f2105ed7bda6f9a882782e337e5f0bcb35406
quarantine: 300 pins, 295 adopted, 5 waiting, 0 refused, 0 already recorded, 0 advice; lockfiles not modified
exit 0
$ cmp Cargo.lock ../Cargo.lock.before && sha256sum Cargo.lock && git status --porcelain Cargo.lock
identical
6ddf54f90058d59d309787a1a92b7050d2a1f4708a49fb877fafe73b9d78ee4e  Cargo.lock
exit 0
$ target/debug/apparatus rws check | tail -4
refusals recorded: 0
open failures: 0
open tickets: 0
check ok
$ target/debug/apparatus rws head
01a0f820-e77b-7271-b89f-cafa2947a864 5b4335aa1fc693315d2baa19e96f2105ed7bda6f9a882782e337e5f0bcb35406 306
```

The five pins that are waiting (younger than 5 days), from the same run:

```text
pin cargo:lazy_static@1.5.1 waiting (age 0d < 5d); adopted stays none
pin cargo:quinn-proto@0.11.19 waiting (age 1d < 5d); adopted stays none
pin cargo:quinn-udp@0.5.16 waiting (age 1d < 5d); adopted stays none
pin cargo:tokio-rustls@0.26.6 waiting (age 3d < 5d); adopted stays none
pin cargo:yoke-derive@0.8.4 waiting (age 1d < 5d); adopted stays none
```

"Waiting" is recorded on the chain. It does not block the build and it does not change the lockfile.

## Repeat it

```bash
mkdir run && cd run
git clone https://github.com/infraax/Aparatus && git -C Aparatus checkout 2e359581cfff99e23874345d2bdba2474771d175
git clone https://github.com/infraax/NAP-corpus && git -C NAP-corpus checkout fb3a312cd84f79fb2446247f5d5dfec260330438
(cd Aparatus && scripts/ic-up.sh && scripts/dev-up.sh)
cd NAP-corpus/docs/ic/canisters
for c in status-notary policy-book; do (cd $c && npm ci && npx mops install && mkdir -p build &&
  $(npx mops toolchain bin moc) $(npx mops sources) -o build/v2.wasm src/v2.mo); done
(cd tools/ic-install && cargo build --locked)
export IC_INSTALL_KEY=$PWD/../../../../Aparatus/.ic-local/dev.key
I=tools/ic-install/target/debug/ic-install
N=$($I create | awk '{print $2}'); $I install $N status-notary/build/v2.wasm; $I record $N 1 healthy
P=$($I create | awk '{print $2}'); $I install $P policy-book/build/v2.wasm
$I pb-advise $P "advice"; $I pb-propose $P 0 build; $I pb-enact $P 0   # exit 2
cd ../../../../Aparatus
target/debug/apparatus rws ic status
scripts/ic-down.sh && scripts/ic-up.sh && ../NAP-corpus/docs/ic/canisters/$I pb-show $P
cp Cargo.lock ../Cargo.lock.before && scripts/quarantine.sh && cmp Cargo.lock ../Cargo.lock.before
scripts/dev-down.sh && scripts/ic-down.sh
```
