## Release — `dev` → `main`

> **This PR merges `dev` into `main` and triggers a release.**  
> Verify every item below before merging. Use squash-merge to keep `main` history clean.

---

## Version

<!-- State the new version number: e.g. `0.4.0` -->

**New version:** 

<!-- Bump checklist — run `just bump <version>` or update manually: -->
- [ ] `core/Cargo.toml` version updated
- [ ] `bindings/python/Cargo.toml` version updated
- [ ] All other binding manifests updated (if applicable)

## Changelog

<!-- Paste or link the relevant CHANGELOG.md section for this release. -->

<details>
<summary>What's new</summary>

<!-- e.g.:
### Added
- `Image.save()` — save to path or buffer with optional format override
- `decode()` and `guess_format()` now accept file-like objects

### Changed
- JPEG encoding switched to mozjpeg for PIL-compatible output quality

### Fixed
- Thread-safety crash in pdfium renderer under concurrent test runners
-->

</details>

## Breaking changes

<!-- List any breaking API changes, or write "None". -->

None

## Pre-release checklist

- [ ] All CI checks pass on `dev`
- [ ] `CHANGELOG.md` is up to date
- [ ] `THIRD_PARTY_NOTICES` is up to date for any new dependencies
- [ ] Version bumped in all manifests (see above)
- [ ] Wheel builds verified locally (`just python build`) or via CI artefacts
- [ ] No debug / temporary commits in this batch
