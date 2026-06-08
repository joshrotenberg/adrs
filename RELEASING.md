# Releasing adrs

This document describes the automated release flow for adrs maintainers,
how to verify a complete release, and how to recover from failures.

## Automated release flow

Releases are fully automated via two tools:

1. **release-plz** -- opens a "Release PR" on every push to `main` that bumps
   `Cargo.toml` versions and updates `CHANGELOG.md` based on conventional
   commits. Merging that PR triggers step 2.

2. **cargo-dist** -- when `release-plz` merges its PR it also pushes a version
   tag (e.g. `v0.7.4`). The tag triggers `.github/workflows/release.yml`, which
   uses `cargo-dist` to build all platform artifacts, upload them, and publish
   the GitHub Release and the Homebrew formula.

### End-to-end flow

```
push commits to main
  -> release-plz runs (release-plz.yml)
  -> release-plz opens/updates a "Release PR"

maintainer merges the Release PR
  -> release-plz pushes a version tag (requires COMMITTER_TOKEN)
  -> release.yml triggers on the tag

release.yml jobs:
  plan          -- runs `dist plan` (computes targets/manifest; does NOT create
                   the release)
  build-local-artifacts  -- builds per-platform binaries (5 targets)
  build-global-artifacts -- builds installers (shell, powershell) and checksums
  host          -- uploads artifacts and creates the GitHub Release ONCE, with all
                   artifacts attached. Sole release creator. Idempotent: a re-run
                   uploads to the existing release instead of failing (#283).
  verify-release -- confirms the published release has its full asset set + a valid
                   checksum; re-drafts and fails if incomplete (#285)
  publish-homebrew-formula -- pushes the .rb formula to joshrotenberg/homebrew-brew
  announce      -- no-op placeholder; runs only after verify-release succeeds
  cleanup-on-failure -- drafts the release if any build OR host job fails (#242, #281)
```

Note: a CI/workflow-only change merged to `main` does NOT trigger a release
(release-plz only cuts a release for crate version bumps, #287). To validate a
pipeline change end-to-end, bundle it with a crate change, or push the next
version normally -- the fixed workflow runs on the next tag.

### Target platforms

cargo-dist builds for five targets (configured in `dist-workspace.toml`):

- `aarch64-apple-darwin` (Apple Silicon Mac)
- `x86_64-apple-darwin` (Intel Mac)
- `aarch64-unknown-linux-gnu` (Linux ARM64)
- `x86_64-unknown-linux-gnu` (Linux x86-64)
- `x86_64-pc-windows-msvc` (Windows x86-64)

## Verifying a complete release

A complete release produces the following artifacts attached to the GitHub Release:

- Five platform tarballs (one per target above): `adrs-v<VERSION>-<target>.tar.gz`
- One SHA256 checksum file: `adrs-v<VERSION>-<target>.tar.gz.sha256` per tarball
  (or a single `dist-manifest.json` containing all checksums)
- Shell installer: `adrs-installer.sh`
- PowerShell installer: `adrs-installer.ps1`
- Homebrew formula: committed to `joshrotenberg/homebrew-brew` as `Formula/adrs.rb`
- `dist-manifest.json`: the dist manifest for the release

To verify:

```bash
# Check the GitHub Release page
gh release view v<VERSION> --repo joshrotenberg/adrs

# List all attached assets
gh release view v<VERSION> --repo joshrotenberg/adrs --json assets --jq '[.assets[].name] | sort'

# Verify Homebrew formula was updated
gh api repos/joshrotenberg/homebrew-brew/commits --jq '.[0].commit.message'
```

Expected: the release should have at least 12-14 assets (5 tarballs + 5 checksums
+ 2 installers + `dist-manifest.json`).

## Required secrets

The release workflow requires two secrets configured in the repository settings:

| Secret | Purpose |
|--------|---------|
| `COMMITTER_TOKEN` | A GitHub PAT with `repo` scope. Used by release-plz to push the version tag (GITHUB_TOKEN actions don't trigger other workflows) and by publish-homebrew-formula to push to `joshrotenberg/homebrew-brew`. |
| `CARGO_REGISTRY_TOKEN` | An API token from crates.io. Used by release-plz to publish updated crates (`adrs-core` and `adrs`) to crates.io. |

Both secrets must be set at `https://github.com/joshrotenberg/adrs/settings/secrets/actions`.

## Recovery: incomplete release (missing artifacts)

If the release workflow fails mid-run, the `cleanup-on-failure` job converts the
GitHub Release to a draft so an incomplete release is never publicly visible (it
fires on any build OR host failure). The `verify-release` job additionally
re-drafts and fails if the published release is missing assets. See #242, #281, #285.

To recover:

1. Investigate the failed job in the Actions tab.
2. Fix the root cause (e.g. a dependency issue, a flaky runner).
3. Re-run the failed jobs from the Actions UI ("Re-run failed jobs"). The
   "Create GitHub Release" step is idempotent (#283): a re-run uploads artifacts
   to the existing/drafted release and un-drafts it, instead of failing with
   "a release with the same tag name already exists".
4. Once all jobs pass (including `verify-release`), confirm the release artifacts.

If a re-run fails with `HTTP 403: Resource not accessible by integration`, the
re-run got a restricted `GITHUB_TOKEN` (a known GitHub re-run quirk). Recover
manually with your own `gh` auth instead: download the run's built artifacts and
attach them to the release.

   ```bash
   gh run download <run-id> -D /tmp/rel && cd /tmp/rel
   gh release create v<VERSION> --repo joshrotenberg/adrs --verify-tag \
     --title "adrs <VERSION>" --notes-file <notes> \
     artifacts-build-local-*/adrs-* artifacts-build-global/adrs-installer.* \
     artifacts-build-global/adrs.rb artifacts-build-global/sha256.sum
   # or, if the (drafted) release already exists:
   gh release edit v<VERSION> --repo joshrotenberg/adrs --draft=false
   ```

If you need to delete and recreate a broken release entirely:

```bash
# Delete the broken release (keeps the tag)
gh release delete v<VERSION> --repo joshrotenberg/adrs --yes

# Optionally delete the tag if you need to re-tag
git push --delete origin v<VERSION>

# Then re-push the tag to re-trigger the release workflow
git tag v<VERSION> <commit-sha>
git push origin v<VERSION>
```

## Recovery: stuck release-plz PR

release-plz opens a single "Release PR" per workspace version. It updates
the PR in place on each push to `main`. Common issues:

### PR not opening or not updating

- Check the `release-plz.yml` workflow run logs for errors.
- Verify the `COMMITTER_TOKEN` secret is set and not expired.
- Check that `CARGO_REGISTRY_TOKEN` is valid (release-plz validates it on dry-run).

### PR has a merge conflict

release-plz manages `Cargo.toml` version fields and `CHANGELOG.md`. If
another PR touching those files merges while the release-plz PR is open,
close the release-plz PR and let release-plz open a fresh one on the next
push to `main` (or trigger `release-plz.yml` via `workflow_dispatch`).

### Clippy or CI failure blocking the release PR

The release PR goes through the same CI checks as any other PR. If CI is
red, fix the issue in a separate PR, merge it to `main`, and then let
release-plz update its PR (or close and re-open it via `workflow_dispatch`).

See issue #221 (clippy 1.96 fix) and PR #212 for a real example of a blocked
release PR and its resolution.

### Manual version bump (skip release-plz)

If you need to release without release-plz (e.g. emergency patch):

1. Update `version` in `Cargo.toml` (workspace root and all crate `Cargo.toml` files).
2. Update `CHANGELOG.md` manually.
3. Commit and push to `main`.
4. Push the tag manually:

   ```bash
   git tag v<VERSION>
   git push origin v<VERSION>
   ```

5. Monitor the release workflow.
