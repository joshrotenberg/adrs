# Operations Skills

Skills for AI agents using `adrs` to manage Architecture Decision Records.

## Overview

These skills help AI agents:
- Create and manage ADRs
- Search and analyze decisions
- Maintain decision relationships
- Generate documentation

## Skills

### Create ADR

**Trigger**: New architectural decision needed

**Actions**:
1. Search for related existing ADRs
2. Create ADR with `create_adr` tool
3. Link to related decisions
4. Set appropriate tags

**Parameters**:
```json
{
  "title": "Decision title",
  "context": "Why is this decision needed?",
  "decision": "What was decided?",
  "consequences": "What are the implications?",
  "supersedes": null
}
```

**Note**: Always creates with `proposed` status for human review.

### Search Decisions

**Trigger**: Questions about existing decisions

**Actions**:
1. Use `search_adrs` with relevant query
2. Filter by status if needed
3. Get full content with `get_adr`
4. Check relationships with `get_related_adrs`

**Example Queries**:
- "database decisions" - Search for database-related ADRs
- "authentication" - Find auth-related decisions
- "superseded" - Find replaced decisions

### Update Status

**Trigger**: Decision status change needed

**Actions**:
1. Validate ADR exists with `get_adr`
2. Update status with `update_status`
3. If superseding, link to new ADR

**Status Values**:
- `proposed` - Under discussion
- `accepted` - Approved
- `deprecated` - No longer recommended
- `superseded` - Replaced by another

### Analyze Repository

**Trigger**: Overview of decisions needed

**Actions**:
1. Get repository info with `get_repository_info`
2. List all ADRs with `list_adrs`
3. Group by status
4. Identify unresolved proposals

### Generate Documentation

**Trigger**: Documentation update needed

**Commands**:
```sh
# Table of contents
adrs generate toc > docs/src/reference/adrs/README.md

# Dependency graph
adrs generate graph | dot -Tsvg > graph.svg
```

### Validate Repository

**Trigger**: Check repository health

**Actions**:
1. Run `adrs doctor` or use `validate_adr` for each
2. Check for:
   - Broken links
   - Invalid frontmatter
   - Numbering gaps
3. Report issues with severity
