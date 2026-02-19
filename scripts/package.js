import { execSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.join(__dirname, '..');
const srcTauri = path.join(rootDir, 'src-tauri');
const packageJsonPath = path.join(rootDir, 'package.json');

// Read Version
const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
const version = packageJson.version;

// Check prerequisites
try {
    execSync('vpk --help', { stdio: 'ignore' });
} catch {
    console.error('❌ Error: "vpk" is not installed.');
    console.error('   Please install it using: dotnet tool install -g vpk');
    process.exit(1);
}

// Resolve Icon Path
let iconPath = path.join(srcTauri, 'icons', 'icon.ico');
if (!fs.existsSync(iconPath)) {
    console.warn(`⚠️  Tauri icon not found at ${iconPath}, checking static...`);
    iconPath = path.join(rootDir, 'static', 'favicon.ico');
}

console.log(`🚀 Packaging FeedMee v${version} with Velopack...`);

// 1. Compile Application (Frontend + Backend)
// We uses 'tauri build' to ensure build.rs runs, sidecars/resources are bundled,
// and frontend is built via the beforeBuildCommand.
console.log('🏗️  Compiling binary...');
try {
    execSync('bunx tauri build --no-bundle', { stdio: 'inherit', cwd: rootDir });
} catch {
    console.error('❌ Compilation failed.');
    process.exit(1);
}

// 2. Package with Velopack
const outDir = path.join(rootDir, 'releases');
if (!fs.existsSync(outDir)) fs.mkdirSync(outDir);

// Create clean distribution directory
const distDir = path.join(rootDir, 'dist');
if (fs.existsSync(distDir)) {
    fs.rmSync(distDir, { recursive: true, force: true });
}
fs.mkdirSync(distDir, { recursive: true });

// Copy only the essential files to dist directory
const sourceExe = path.join(srcTauri, 'target', 'release', 'feedmee.exe');
const targetExe = path.join(distDir, 'feedmee.exe');
fs.copyFileSync(sourceExe, targetExe);

console.log('📦 Creating Release Artifacts...');
try {
    const cmd = [
        'vpk pack',
        '--packId FeedMee',
        `--packVersion ${version}`,
        `--packDir "${distDir}"`,
        '--mainExe feedmee.exe',
        `--icon "${iconPath}"`,
        `--outputDir "${outDir}"`,
    ].join(' ');

    execSync(cmd, { stdio: 'inherit', cwd: distDir });

    // Add .portable marker to portable build
    const portableZip = path.join(outDir, `FeedMee-${version}-Portable.zip`);
    if (fs.existsSync(portableZip)) {
        console.log('📝 Adding .portable marker to portable build...');
        const tempExtractDir = path.join(outDir, 'temp_portable');

        try {
            // Create temp directory
            fs.mkdirSync(tempExtractDir, { recursive: true });

            // Extract portable zip
            execSync(`tar -xf "${portableZip}" -C "${tempExtractDir}"`, { stdio: 'inherit' });

            // Add .portable marker file
            fs.writeFileSync(
                path.join(tempExtractDir, '.portable'),
                'This file indicates portable mode',
            );

            // Re-create zip with marker
            fs.rmSync(portableZip);
            execSync(`tar -a -c -f "${portableZip}" -C "${tempExtractDir}" .`, {
                stdio: 'inherit',
            });

            // Clean up temp directory
            fs.rmSync(tempExtractDir, { recursive: true, force: true });

            console.log('✅ Portable marker added successfully!');
        } catch (err) {
            console.warn(
                '⚠️  Failed to add portable marker:',
                err instanceof Error ? err.message : String(err),
            );
        }
    }

    console.log('\n✅ Build Successful!');
    console.log(`   📂 Artifacts Location: ${path.resolve(outDir)}`);
    console.log(`   - Installer: FeedMee-Setup.exe`);
    console.log(`   - Portable:  FeedMee-${version}-Portable.zip (with .portable marker)`);
} catch (error) {
    console.error('❌ Velopack packaging failed.');
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
}
