# UI Package Scripts

## `update-icon-names.ts`

A utility script to update icon names across the codebase when icons are renamed or migrated.

### Usage

```bash
# From the UI package
cd packages/ui
pnpm update-icons

# Or from the root
pnpm update-icons --filter @gitbutler/ui
```

### Configuration

To update icon names, edit the `iconMapping` object in `scripts/update-icon-names.ts`:

```typescript
const iconMapping: Record<string, string> = {
	"old-icon-name": "new-icon-name",
	"another-old-name": "another-new-name",
};
```

### What it does

1. Searches for all occurrences of the old icon names in:
   - TypeScript/JavaScript/Svelte files in `apps/`, `packages/`, `crates/`, and `e2e/`
   - String literals and identifiers
   - Type unions and object properties

2. Replaces them with the new icon names while preserving:
   - Quote styles (single/double)
   - Context and surrounding code
   - File integrity

3. Reports which files were modified

### Example

If you want to migrate from old icon names to new icon names:

```typescript
const iconMapping: Record<string, string> = {
	"branch-remote": "branch",
	"branch-shadow-commit": "branch-double-commit",
};
```

Then run:

```bash
pnpm update-icons
```

The script will find and replace all occurrences throughout the codebase.
