# ContextPacker Guidelines

ContextPacker is an extension for VS Code that provides a wide range of sorting, case change, and line manipulation functions via context menu and command palette. It aims for fast performance, minimal resource usage, fast startup time in VS Code.

## Tech Stack

- ?

## Entry Points

- ?

### Core Components

- ?

### Build Output

- ?

## Key Architecture

### Performance Optimizations

- ?

### Caching Strategy

- ?

### Event Handling

- ?

### Decorator System

- ?

## Coding Principles

- Use current coding standards and patterns
- KISS, Occam's razor, DRY, YAGNI
- Optimize for actual and perceived performance
- Self-documenting code via clear naming
- Comments only for workarounds/complex logic
- No magic numbers - use constants like `CHUNK_SIZE_LINES`, `VISIBLE_LINE_BUFFER`
- **Do NOT create docs files** (summary, reference, testing, etc.) unless explicitly instructed

## File System Access

### Allowed

- `.agents/`, `.github/`, `.vscode/`
- `scripts/`, `src/`
- Root files: `README.md`, `.editorconfig`, `.gitignore`, `eslint.config.mjs`, `package.json`, `tsconfig.json`, etc.

### Disallowed

- `.assets/`, `.docs/`, `.git/`, `node_modules/`
- `repomix.config.json`, `.repomixignore`

## Common Patterns

- ?
