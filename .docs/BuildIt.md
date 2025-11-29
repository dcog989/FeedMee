# Build It


## Daily Development

Start the Vite frontend server and the Rust backend with Hot Module Replacement (HMR):

```powershell
npm run tauri dev
```

## Production Build

This compiles the Rust backend in release mode, bundles the Svelte frontend, and generates the installer (`.msi` / `.exe`) in `src-tauri/target/release/bundle/nsis/`:

```powershell
npm run tauri build
```

## Code Quality (Type Checking)

Since you are using TypeScript and Svelte 5, run this to check for type errors without building the whole app:

```powershell
# Run once
npm run check

# Watch mode (keep running in a separate terminal)
npm run check:watch
```

## Cleaning (Troubleshooting)

If the app misbehaves, use these commands to wipe caches:

**Option A: The Manual Way**

```powershell
# 1. Clean Rust artifacts (Backend)
cd src-tauri
cargo clean
cd ..

# 2. Clean Node dependencies (Frontend)
Remove-Item -Recurse -Force node_modules
Remove-Item -Force package-lock.json

# 3. Reinstall
npm install
```

**Option B: Your Custom Script**

Use script `.scripts/buildit.ps1` that handles this for you with a menu interface.
```powershell
.\.scripts\buildit.ps1
```
*(Select option **3** to "Clean & Install")*

## Testing
*Currently, you do not have a dedicated test runner (like Vitest) configured in `package.json`, so standard testing commands won't work yet.* 

For now, use `npm run check` to validate code integrity.