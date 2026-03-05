# UX Requirements

<!-- toc -->

## Error Messages

### CLI-UX-1: Actionable Errors

**Requirements:**
- Error messages MUST be actionable
- Error messages MUST include context (file, line)
- Error messages SHOULD suggest solutions

**Example:**
```
Error: No ADR configuration found

  Hint: Run 'adrs init' to create one.
```

### CLI-UX-2: Error Format

**Requirements:**
- Errors MUST go to stderr
- Errors MUST use non-zero exit code
- Errors SHOULD be colorized when terminal supports it

## Editor Integration

### CLI-UX-3: Editor Selection

**Requirements:**
- MUST respect `$EDITOR` environment variable
- MUST fall back to common editors (vim, nano, vi)
- MUST handle editor exit codes (0 = save, non-0 = cancel)

### CLI-UX-4: Editor Behavior

**Requirements:**
- MUST open file at correct position
- MUST wait for editor to close
- MUST save changes on normal exit
- MUST discard on error exit

## Shell Completions

### CLI-UX-5: Completion Support

**Requirements:**
- MUST support bash, zsh, fish, PowerShell
- MUST complete command names
- MUST complete ADR numbers where applicable
- MUST complete status values

### CLI-UX-6: Completion Generation

```sh
adrs completions bash > ~/.local/share/bash-completion/completions/adrs
adrs completions zsh > ~/.zfunc/_adrs
adrs completions fish > ~/.config/fish/completions/adrs.fish
```

## Output

### CLI-UX-7: Human-Readable Output

**Requirements:**
- Default output MUST be readable without tools
- Tables SHOULD align columns
- Lists SHOULD use consistent formatting

### CLI-UX-8: Machine-Readable Output

**Requirements:**
- `--json` MUST output valid JSON
- JSON MUST be parseable by standard tools
- JSON SHOULD match documented schema

### CLI-UX-9: Progress Indication

**Requirements:**
- Long operations SHOULD show progress
- Batch operations SHOULD show count

## Confirmation

### CLI-UX-10: Destructive Actions

**Requirements:**
- Destructive actions SHOULD require confirmation
- Confirmation MAY be skipped with `-y` or `--yes`
- Confirmation MUST be skipped in non-interactive mode

## See Also

- [Command Requirements](./commands.md)
- [Commands Reference](../../../users/commands/README.md)
