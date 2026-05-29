import { execSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.join(__dirname, '..');
const packageJsonPath = path.join(rootDir, 'package.json');

const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
const version = packageJson.version;

console.log(`🚀 Packaging FeedMee v${version}...`);

try {
    execSync('bunx tauri build', { stdio: 'inherit', cwd: rootDir });
} catch {
    console.error('❌ Build failed.');
    process.exit(1);
}

const bundleDir = path.join(rootDir, 'src-tauri', 'target', 'release', 'bundle');
const outDir = path.join(rootDir, 'releases');
fs.mkdirSync(outDir, { recursive: true });

const artifactDirs = {
    appimage: path.join(bundleDir, 'appimage'),
};

let copied = 0;
for (const [type, dir] of Object.entries(artifactDirs)) {
    if (!fs.existsSync(dir)) continue;
    for (const file of fs.readdirSync(dir)) {
        fs.copyFileSync(path.join(dir, file), path.join(outDir, file));
        console.log(`   ✅ ${type}: ${file}`);
        copied++;
    }
}

if (copied === 0) {
    console.warn('⚠️  No artifacts found. Check that bundle.active is true in tauri.conf.json.');
    process.exit(1);
}

console.log(`\n✅ Done. Artifacts in: ${path.resolve(outDir)}`);
