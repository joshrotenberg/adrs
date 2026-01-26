# cheatsheet

Show a quick reference for common ADR workflows.

## Usage

```
adrs cheatsheet
```

Alias: `adrs qr`

## Output

Displays a comprehensive quick reference covering:

- Getting started
- Creating ADRs
- Superseding and linking
- Managing status
- Viewing and searching
- Generating documentation
- Import/export
- Configuration

## Example Output

```
ADR Quick Reference
===================

GETTING STARTED
  adrs init                    Initialize ADR repository
  adrs --ng init               Initialize with NextGen mode (YAML frontmatter)

CREATING ADRS
  adrs new "Title"             Create new ADR
  adrs new --format madr       Use MADR 4.0.0 format
  adrs new --variant minimal   Use minimal template
  adrs new -t tag1,tag2        Add tags (NextGen mode)
  adrs new --no-edit           Create without opening editor

SUPERSEDING AND LINKING
  adrs new -s 2 "New title"    Supersede ADR #2
  adrs link 3 Amends 1         Link ADR #3 amends #1

MANAGING STATUS
  adrs status 1 accepted       Accept ADR #1
  adrs status 2 deprecated     Deprecate ADR #2
  adrs status 3 superseded 4   Mark #3 superseded by #4

VIEWING AND SEARCHING
  adrs list                    List all ADRs
  adrs list --status accepted  Filter by status
  adrs search postgres         Search content
  adrs search -t database      Search titles only

DOCUMENTATION
  adrs generate toc            Generate table of contents
  adrs generate graph          Generate Graphviz diagram
  adrs generate book           Generate mdbook

IMPORT/EXPORT
  adrs export json             Export to JSON-ADR
  adrs import file.json        Import from JSON-ADR

CONFIGURATION
  adrs config                  Show current config
  adrs doctor                  Check repository health

More: adrs --help, adrs <command> --help
```

## Tips

- Run `adrs cheatsheet` whenever you need a quick reminder
- Use `adrs <command> --help` for detailed command documentation
- Full documentation at [joshrotenberg.github.io/adrs-book](https://joshrotenberg.github.io/adrs-book/)
