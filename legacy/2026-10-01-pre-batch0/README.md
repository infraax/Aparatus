# Legacy snapshot: before Batch 0 (2026-10-01)

An exact copy of the tracked sources before Batch 0 changed them. The source commit is on the first line of `MANIFEST.sha256`.

- Not built, not tested, not imported by anything.
- It exists only so the old code can be read or restored by hand.
- Check it is unchanged with `cd legacy/2026-10-01-pre-batch0 && grep -v '^#' MANIFEST.sha256 | sha256sum -c --quiet`.
- Delete it only on Dex's word.
