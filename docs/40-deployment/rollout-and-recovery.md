# Rollout And Recovery

## Rollout

1. Generate/update schema from typed config when config model changes.
2. Publish Core artifacts for Moira consumption through the GitHub Actions workflow path, either from a version tag push or a manual workflow dispatch that names the release tag explicitly.
3. The first producer workflow currently runs on `macos-14` so it can build the current first supported Moira consumer target `aarch64-apple-darwin`.
4. The workflow must produce the release outputs required by the Product TDD contract:
   - `beluna-core-<rust-target-triple>.tar.gz`
   - `SHA256SUMS`
5. Publish the release or prerelease with matching archive and checksum assets.
6. Verify that the published release includes the current first supported Moira consumer target `aarch64-apple-darwin`.
7. Verify that the archive can be downloaded, checksum-verified, extracted, and activated by Moira without manual path patching.
8. Deploy Core with validated config.
9. Verify endpoint registration and runtime startup telemetry.

## Recovery

1. Use process signals for graceful shutdown.
2. Ensure ingress closure and bounded efferent drain.
3. Restart with corrected config/runtime dependencies.
4. Use persisted cognition state and logs for incident analysis.
5. If Moira reports checksum mismatch, missing `SHA256SUMS`, or missing target asset, treat the release as invalid for consumer activation and republish corrected artifacts.
6. If the workflow publishes a release with the wrong asset names, delete or replace the assets and rerun the workflow rather than teaching Moira a workflow-specific exception.
7. If the published archive extracts without the expected executable or install isolation cannot be completed cleanly, withdraw or replace the release asset instead of teaching Moira a one-off extraction exception.
