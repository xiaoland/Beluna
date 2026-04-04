# Issue 8 Minimal Release Producer Working Set

## Context

`#8` is the producer-side counterpart to Moira's landed release-intake path.
Moira already knows how to discover GitHub Releases, verify `SHA256SUMS`, and install
`beluna-core-<rust-target-triple>.tar.gz` into an isolated local directory.

This working set covers only the minimum release producer slice needed to let Moira
run a live walkthrough against a real GitHub Release.

## Intended Change

1. Add a GitHub Actions workflow that builds the current first supported consumer target
   `aarch64-apple-darwin`.
2. Package the Core executable into the authoritative archive name
   `beluna-core-aarch64-apple-darwin.tar.gz`.
3. Generate a release-level `SHA256SUMS`.
4. Publish or update a GitHub Release / prerelease with those two assets.

## Boundaries

- This task does not expand into a full multi-platform CD matrix.
- This task does not add detached signatures, website deployment, auto-update,
  or staged environment promotion.
- Contract authority remains in `docs/20-product-tdd/*`.
- Operational truth remains in `docs/40-deployment/*`.
- The workflow should call a repository script for packaging so contract details do not
  live only inside workflow YAML.

## Verification

- Local packaging script can build `core`, produce
  `beluna-core-aarch64-apple-darwin.tar.gz`, and emit `SHA256SUMS`.
- Workflow YAML uses the same packaging script and target triple.
- Deployment docs mention the workflow trigger path and recovery expectations.
- `#8` issue body can point to the working set, workflow, and packaging script.
- The current full `core` test baseline is not yet a release gate for this minimal slice;
  that gap should stay explicit rather than silently blocking artifact publication.
