# Verify the repo: frontend and Rust checks, the same set CI runs.
default: check

# Full check: Svelte types, Biome lint, rustfmt, and Clippy.
check:
    bun run check

# Linter alone.
lint:
    bunx biome check .

# Rewrite Rust and frontend formatting.
format:
    bun run format

# Run the frontend (Vitest) and Rust test suites.
test:
    bun run test

# Build the release binary (no installers).
build:
    bun run build

# Tauri dev with hot reload.
dev:
    bun run dev

# makepkg from PKGBUILD.
package:
    bun run package

# Remove build artifacts and node_modules.
clean:
    bun run clean

# Refresh dependencies to their latest allowed versions.
update:
    bun run update
