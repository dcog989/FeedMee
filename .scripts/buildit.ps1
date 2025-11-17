# Get the directory where the script is located (.scripts folder)
$ScriptRoot = $PSScriptRoot

# Define paths relative to the script location
# The project root is now one level up from .scripts
$ProjectRoot = Split-Path $ScriptRoot -Parent
$NodeModulesPath = Join-Path $ProjectRoot "node_modules"
$LockFilePath = Join-Path $ProjectRoot "package-lock.json"
$TauriDir = Join-Path $ProjectRoot "src-tauri"

Write-Host "[DEBUG] Script Root (`.scripts` dir): $ScriptRoot"
Write-Host "[DEBUG] Calculated Project Root: $ProjectRoot"

function Show-Menu {
    Clear-Host
    Write-Host "================ FeedMee Project Manager ================"
    Write-Host "1: Install Dependencies"
    Write-Host "2: Clean Project (npm & Rust)"
    Write-Host "3: Clean & Install"
    Write-Host "4: Run Development Server"
    Write-Host "Q: Quit"
    Write-Host "========================================================="
}

function Test-BlockingProcesses {
    while ($true) {
        $blockingProcesses = Get-Process -Name "Code", "devenv" -ErrorAction SilentlyContinue
        if ($null -eq $blockingProcesses) {
            return $true # No blocking processes found
        }

        Write-Host "`n[WARN] The following applications are running which can lock files:" -ForegroundColor Yellow
        foreach ($proc in $blockingProcesses) {
            Write-Host "- $($proc.ProcessName)" -ForegroundColor Yellow
        }
        $userInput = Read-Host "Please close them and press Enter to continue, or type 'S' to skip this check"
        if ($userInput -eq 's') {
            Write-Host "[INFO] Skipping process check..." -ForegroundColor Cyan
            return $true
        }
    }
}

function Clear-ProjectBuildCache {
    Write-Host "`n[INFO] Starting project clean..."

    if (-not (Test-BlockingProcesses)) {
        Write-Host "[ERROR] Clean operation cancelled by user." -ForegroundColor Red
        return
    }

    # Clean Rust target directory
    if (Test-Path $TauriDir) {
        Write-Host "- Running 'cargo clean' in '$TauriDir'..."
        Push-Location $TauriDir
        try {
            cargo clean
        }
        catch {
            Write-Host "[ERROR] An error occurred while running 'cargo clean': $_" -ForegroundColor Red
        }
        finally {
            Pop-Location
        }
    }
    else {
         Write-Host "[WARN] 'src-tauri' directory not found. Skipping 'cargo clean'." -ForegroundColor Yellow
    }

    # Clean npm artifacts
    if (Test-Path $NodeModulesPath) {
        Write-Host "- Removing node_modules..."
        # Use cmd's rmdir as it's more resilient to file locks than PowerShell's Remove-Item
        cmd.exe /c "rmdir /s /q `"$NodeModulesPath`""
    }
    if (Test-Path $LockFilePath) {
        Write-Host "- Removing package-lock.json..."
        Remove-Item -Force $LockFilePath
    }
    Write-Host "[INFO] Clean complete."
}

function Install-Dependencies {
    Write-Host "`n[INFO] Installing dependencies in '$ProjectRoot'..."
    if (Test-Path $ProjectRoot) {
        Push-Location $ProjectRoot
        npm install
        Pop-Location
        Write-Host "[SUCCESS] Installation complete."
    }
    else {
        Write-Host "[ERROR] Project root directory not found: $ProjectRoot" -ForegroundColor Red
    }
}

while ($true) {
    Show-Menu
    $choice = Read-Host "Please enter your choice"

    switch ($choice) {
        "1" {
            Install-Dependencies
            Write-Host "Press any key to continue..."
            $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
        }
        "2" {
            Clear-ProjectBuildCache
            Write-Host "Press any key to continue..."
            $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
        }
        "3" {
            Clear-ProjectBuildCache
            Install-Dependencies
            Write-Host "Press any key to continue..."
            $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
        }
        "4" {
            Write-Host "`n[INFO] Starting the Tauri development server from '$ProjectRoot'..."
            Write-Host "[INFO] This will take over the console. Close the app window to stop."
            if (Test-Path $ProjectRoot) {
                Push-Location $ProjectRoot
                try {
                    npm run tauri dev
                }
                catch {
                    Write-Host "[ERROR] An error occurred while running 'npm run tauri dev': $_" -ForegroundColor Red
                }
                finally {
                    Pop-Location
                }
            }
            else {
                Write-Host "[ERROR] Project root directory not found: $ProjectRoot" -ForegroundColor Red
                Write-Host "Please ensure the path is correct." -ForegroundColor Red
            }
            Write-Host "[SUCCESS] Development server stopped. Press any key to continue..."
            $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
        }
        "q" {
            Write-Host "Exiting."
            return
        }
        default {
            Write-Host "`n[ERROR] Invalid option. Press any key to try again..." -ForegroundColor Red
            $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
        }
    }
}