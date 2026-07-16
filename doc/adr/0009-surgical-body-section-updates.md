# 9. Surgical body section updates preserve on-disk markdown

Date: 2026-07-11

## Status

Proposed

## Context

`Repository::update` and MCP tools such as `update_content` and `create_adr` follow a read-modify-write path: load the ADR, mutate in-memory fields, then write the file back. For body sections, the write path previously re-rendered every canonical section from the in-memory `Adr` struct.

That approach is unsafe because `parse_sections` is intentionally lossy when reading markdown. It flattens pulldown-cmark events to plain text, which drops structure that ADRs commonly contain:

- MADR 4.0.0 H3 subsections under `## Decision Outcome` (`### Consequences`, `### Confirmation`, and others)
- Bulleted and numbered lists
- Links and inline code
- Optional MADR H2 sections such as `## Considered Options` and `## Pros and Cons of the Options`

A context-only MCP update therefore corrupted untouched sections: Decision Outcome subsections disappeared, bullets flattened, and links lost their markdown syntax. The same bug affected NextGen Nygard ADRs and legacy adr-tools files, not only MADR 4.0.0.

Metadata-only updates already had a separate, byte-preserving path (`update_metadata`). Body updates did not.

## Decision

Patch only the body sections the caller explicitly requests to change. Leave all other on-disk bytes untouched.

1. **`BodySectionPatch` API** — `Repository::update(adr, patch)` takes optional `context`, `decision`, and `consequences` fields. `None` means do not rewrite that section. An empty patch runs metadata-only updates via the same path as `update_metadata`.

2. **Metadata/body split** — When `body` is non-empty, `update()` patches body sections only and does not rewrite metadata bytes (status, links, tags, or frontmatter people fields). MCP `create_adr` calls `update_metadata` and `update` separately when both are needed.

3. **No lossy round-trip for unmodified sections** — Never source write content from reparsed `Adr` fields for sections absent from the patch, even if those fields were populated by `get()` or `parse()`.

4. **Fence-aware scanners** — Body section patching ignores `##` / `###` lines inside markdown code fences when locating section boundaries.

5. **MADR `## Decision Outcome`** — When `decision` is patched, replace only the intro text before the first `###` heading outside fences. Preserve all other H3 subsections verbatim.

6. **Consequences routing**:
   - When a top-level `## Consequences` H2 exists anywhere outside fences (before or after Decision), patch that H2 and do not inject `### Consequences` under Decision Outcome / Decision.
   - **MADR** (no Consequences H2): Patch the `### Consequences` subsection under `## Decision Outcome`. If that subsection does not exist, append `### Consequences` under Decision Outcome.
   - **Nygard / adr-tools** (no Consequences H2): return an error (do not append under `## Decision`).

7. **People-field YAML** — Frontmatter metadata writes parse the YAML block into a `Mapping`, mutate managed keys (`status`, `links`, `tags`, people fields), and re-emit. String-or-list people/tag values that already match semantically are left untouched so block scalars and unusual list indent survive no-op and status/link updates.

8. **Missing section headings** — `update()` returns an error when a patch field is set but no matching section heading exists on disk (including Nygard consequences without a `## Consequences` H2).

9. **Unknown H2 sections** — Sections that are not canonical context/decision/consequences headings are always copied verbatim from the existing file.

10. **Caller contract** — MCP `update_content` builds `BodySectionPatch` from only the parameters the caller supplied, not from the full in-memory ADR after `get()`. MCP `update_content` rejects calls where all of `context`, `decision`, and `consequences` are omitted.

This logic lives in `adrs-core` (see ADR 4). It applies in both compatible and NextGen repositories and to on-disk files in either Nygard or MADR layout (see ADR 5). MCP is a primary caller (see ADR 8).

## Consequences

- Untouched sections remain byte-identical on disk, including rich markdown, fenced examples, and MADR-specific structure
- Body-only MCP `update_content` no longer rewrites legacy `## Status` prose or frontmatter people fields
- Callers must pass `BodySectionPatch`; an empty patch updates metadata only and does not re-render body text from in-memory `Adr` fields. `BodySectionPatch` is `#[non_exhaustive]`, so out-of-crate construction uses `new()` / `with_*` (or field assignment after `Default`).
- The parser may remain lossy for reads, search, and display; correctness depends on disciplined write paths
- **`validate_adr` false positives on MADR 4.0.0** remain a read/validation gap (out of scope)
- YAML comments in frontmatter are not preserved when any managed metadata field changes (Mapping re-emit)
- Deferred follow-ups: dual-mode `update()` split (#341)
- CRLF preservation through the write path (#339) and the blank-line separator after replaced section bodies (#340) are fixed: `update_body_sections` and `update_frontmatter_metadata` detect the file's dominant line ending and re-emit every line, both copied-through and patched, with that ending, and a patched section is followed by exactly one blank line before the next heading (none at EOF, so repeated identical patches are byte-identical)
- MADR `### Consequences` read round-trip is fixed (#338): `parse_sections` and `extract_sections_raw` now route the H3 under `## Decision Outcome` / `## Decision` to `consequences` instead of folding it into `decision`, so `get_adr_sections`, `compare_adrs`, and `validate_adr` see the same sections the write path targets
