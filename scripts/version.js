import { execSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

// Get the directory of the current script
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.join(__dirname, '..');
const packageJsonPath = path.join(rootDir, 'package.json');
const tauriConfPath = path.join(rootDir, 'src-tauri', 'tauri.conf.json');
const cargoTomlPath = path.join(rootDir, 'src-tauri', 'Cargo.toml');

// Parse arguments
const args = process.argv.slice(2);
const shouldGit = args.includes('--git');
const versionArg = args.find((arg) => !arg.startsWith('--'));

// 1. Read current version from package.json
/** @type {string} */
let currentVersion;
/** @type {{version: string}} */
let packageJson;

try {
    packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
    currentVersion = packageJson.version;
} catch (error) {
    console.error('Failed to read package.json:', error);
    process.exit(1);
}

// 2. Determine new version
/** @type {string} */
let newVersion = versionArg || '';

if (!newVersion) {
    // No argument provided: Auto-increment patch
    const parts = currentVersion.split('.').map((n) => parseInt(n, 10));

    if (parts.length !== 3 || parts.some(isNaN)) {
        console.error(
            `Error: Current version '${currentVersion}' is not in semver format (x.y.z). Cannot auto-increment.`,
        );
        process.exit(1);
    }

    parts[2] += 1; // Increment patch
    newVersion = parts.join('.');
    console.log(`Auto-incrementing patch: ${currentVersion} -> ${newVersion}`);
} else {
    // Argument provided
    console.log(`Manual override: ${currentVersion} -> ${newVersion}`);
}

// Validate semver format
if (!/^\d+\.\d+\.\d+/.test(newVersion)) {
    console.error(`Error: New version '${newVersion}' must be in format x.y.z`);
    process.exit(1);
}

// 3. Update package.json
try {
    packageJson.version = newVersion;
    fs.writeFileSync(packageJsonPath, JSON.stringify(packageJson, null, 2) + '\n');
    console.log('✅ Updated package.json');
} catch (error) {
    console.error('Failed to update package.json:', error);
    process.exit(1);
}

// 4. Update tauri.conf.json
try {
    const content = JSON.parse(fs.readFileSync(tauriConfPath, 'utf8'));
    content.version = newVersion;
    fs.writeFileSync(tauriConfPath, JSON.stringify(content, null, 2) + '\n');
    console.log('✅ Updated src-tauri/tauri.conf.json');
} catch (error) {
    console.error('Failed to update tauri.conf.json:', error);
    process.exit(1);
}

// 5. Update Cargo.toml
try {
    let content = fs.readFileSync(cargoTomlPath, 'utf8');
    const regex = /(\[package\][\s\S]*?^version = ")([^"]+)(")/m;

    if (regex.test(content)) {
        content = content.replace(regex, `$1${newVersion}$3`);
        fs.writeFileSync(cargoTomlPath, content);
        console.log('✅ Updated src-tauri/Cargo.toml');
    } else {
        console.error('❌ Could not find [package] version string in Cargo.toml');
        process.exit(1);
    }
} catch (error) {
    console.error('Failed to update Cargo.toml:', error);
    process.exit(1);
}

// 6. Git Integration
if (shouldGit) {
    try {
        console.log('\n📦 Processing Git operations...');

        // Stage the files
        // We use forward slashes for cross-platform compatibility in exec commands,
        // although path.join handles OS separators, git usually accepts forward slashes.
        // Using strict paths ensures we only add what we changed.
        const files = [packageJsonPath, tauriConfPath, cargoTomlPath]
            .map((p) => `"${p}"`)
            .join(' ');
        execSync(`git add ${files}`, { stdio: 'inherit' });

        // Commit
        const commitMsg = `chore: release v${newVersion}`;
        execSync(`git commit -m "${commitMsg}"`, { stdio: 'inherit' });

        // Tag
        const tagName = `v${newVersion}`;
        execSync(`git tag -a ${tagName} -m "${tagName}"`, { stdio: 'inherit' });

        console.log(`✅ Git commit and tag '${tagName}' created successfully`);
    } catch (error) {
        console.error(
            '\n❌ Git operations failed. The files were updated, but git actions were skipped.',
        );
        console.error(error instanceof Error ? error.message : String(error));
        // We don't exit(1) here because the primary bump operation succeeded.
    }
} else {
    console.log(`\n🎉 Successfully updated version to v${newVersion}`);
    console.log('   (Run with --git to automatically commit and tag)');
}
