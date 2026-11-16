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

# package.json script: check:frontend.
check-frontend:
    bun run check:frontend

# package.json script: check:types.
check-types:
    bun run check:types

# package.json script: coverage.
coverage:
    bun run coverage

# package.json script: format:check.
format-check:
    bun run format:check

# package.json script: gen-icons.
icons:
    bun run gen-icons

# Bump the version from conventional commits and tag it.
release:
    cog bump --auto

# Full Tauri bundles: AppImage, deb, rpm, NSIS.
bundle:
    bunx tauri build

# Rust backend only: rustfmt and clippy.
check-backend:
    cargo fmt --manifest-path src-tauri/Cargo.toml --check
    cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings

# Rust backend tests.
test-backend:
    cargo test --manifest-path src-tauri/Cargo.toml

# Verify commit messages in a range (`just commits origin/main..HEAD`).
commits range="origin/main..HEAD":
    cog check {{range}}

# Regenerate the SvelteKit types before a type check.
sync:
    bunx svelte-kit sync

# Verify commit messages in a range (`just commits origin/main..HEAD`).
