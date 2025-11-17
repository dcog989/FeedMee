# Get the directory where the script is located (.scripts folder)
$ScriptRoot = $PSScriptRoot

# Define paths relative to the script location
# The project root is now one level up from .scripts
$ProjectRoot = Split-Path $ScriptRoot -Parent
$NodeModulesPath = Join-Path $ProjectRoot "node_modules"
$LockFilePath = Join-Path $ProjectRoot "package-lock.json"

Write-Host "[DEBUG] Script Root (`.scripts` dir): $ScriptRoot"
Write-Host "[DEBUG] Calculated Project Root: $ProjectRoot"

function Show-Menu {
    Clear-Host
    Write-Host "================ FeedMee Project Manager ================"
    Write-Host "1: Install Dependencies (in project root)"
    Write-Host "2: Clean & Install Dependencies (in project root)"
    Write-Host "3: Run Development Server (from project root)"
    Write-Host "Q: Quit"
    Write-Host "========================================================="
}

while ($true) {
    Show-Menu
    $choice = Read-Host "Please enter your choice"

    switch ($choice) {
        "1" {
            Write-Host "`n[INFO] Installing dependencies in '$ProjectRoot'..."
            if (Test-Path $ProjectRoot) {
                Push-Location $ProjectRoot
                npm install
                Pop-Location
                Write-Host "[SUCCESS] Installation complete. Press any key to continue..."
            }
            else {
                Write-Host "[ERROR] Project root directory not found: $ProjectRoot" -ForegroundColor Red
            }
            $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
        }
        "2" {
            Write-Host "`n[INFO] Cleaning project..."
            if (Test-Path $NodeModulesPath) {
                Write-Host "- Removing node_modules..."
                Remove-Item -Recurse -Force $NodeModulesPath
            }
            if (Test-Path $LockFilePath) {
                Write-Host "- Removing package-lock.json..."
                Remove-Item -Force $LockFilePath
            }
            Write-Host "[INFO] Clean complete."

            Write-Host "`n[INFO] Installing dependencies in '$ProjectRoot'..."
            if (Test-Path $ProjectRoot) {
                Push-Location $ProjectRoot
                npm install
                Pop-Location
                Write-Host "[SUCCESS] Clean install complete. Press any key to continue..."
            }
            else {
                Write-Host "[ERROR] Project root directory not found: $ProjectRoot" -ForegroundColor Red
            }
            $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
        }
        "3" {
            Write-Host "`n[INFO] Starting the Tauri development server from '$ProjectRoot'..."
            Write-Host "[INFO] This will take over the console. Close the app window to stop."
            if (Test-Path $ProjectRoot) {
                Push-Location $ProjectRoot # Change to the project root
                try {
                    npm run tauri dev
                }
                catch {
                    Write-Host "[ERROR] An error occurred while running 'npm run tauri dev': $_" -ForegroundColor Red
                }
                finally {
                    Pop-Location # Return to the script's original location or previous location on the stack
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