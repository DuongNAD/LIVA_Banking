<#
.SYNOPSIS
    LIVA Banking Harness -- ngrok Remote Access Tunnel Automation Script.

.DESCRIPTION
    Automates the deployment, lifecycle management, health verification,
    and inspection of the ngrok remote access tunnel for the LIVA Treasury Workbench.
    Enforces RAM pre-flight guardrails (>= 4GB free memory) per AGENTS.md,
    verifies upstream service connectivity, and configures TLS termination.

.PARAMETER Action
    Action to perform: 'test', 'start', 'status', 'verify', 'inspect', 'stop'.
    Default: 'test'.

.PARAMETER Port
    Local upstream port where Treasury Workbench is listening.
    Default: 5173 (Vite dev server). Supported alternatives: 4173 (preview), 8080 (production).

.PARAMETER Domain
    Assigned ngrok domain.
    Default: 'rd_3JHcwHJK1iUTdwx1D22XRMeUA5K.ngrok-free.app'.

.PARAMETER ConfigFile
    Path to ngrok configuration file.
    Default: 'deploy/ngrok/ngrok-treasury-workbench.yml'.

.PARAMETER Foreground
    If set, runs ngrok in the foreground console instead of background process.

.PARAMETER SkipRamCheck
    If set, bypasses the RAM guard pre-flight check (not recommended for production).

.EXAMPLE
    .\scripts\deploy-ngrok-tunnel.ps1 -Action test
    .\scripts\deploy-ngrok-tunnel.ps1 -Action start -Port 5173
    .\scripts\deploy-ngrok-tunnel.ps1 -Action status
    .\scripts\deploy-ngrok-tunnel.ps1 -Action inspect
    .\scripts\deploy-ngrok-tunnel.ps1 -Action stop
#>

[CmdletBinding()]
param (
    [Parameter(Position = 0)]
    [ValidateSet('test', 'start', 'status', 'verify', 'inspect', 'stop', 'help')]
    [string]$Action = 'test',

    [Parameter()]
    [int]$Port = 5173,

    [Parameter()]
    [string]$Domain = 'rd_3JHcwHJK1iUTdwx1D22XRMeUA5K.ngrok-free.app',

    [Parameter()]
    [string]$ConfigFile = 'deploy/ngrok/ngrok-treasury-workbench.yml',

    [Parameter()]
    [switch]$Foreground,

    [Parameter()]
    [switch]$SkipRamCheck,

    [Parameter()]
    [switch]$FallbackEphemeral
)

$ErrorActionPreference = "Stop"

# Paths
$ScriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptRoot
$ResolvedConfigFile = Join-Path $ProjectRoot $ConfigFile
$DeployDir = Join-Path $ProjectRoot "deploy/ngrok"
$PidFile = Join-Path $DeployDir ".ngrok.pid"
$LogFile = Join-Path $DeployDir "ngrok.log"
$RamGuardScript = Join-Path $ProjectRoot "scripts/ram-guard.ps1"

function Show-Header {
    Write-Host ""
    Write-Host "==========================================================================" -ForegroundColor Cyan
    Write-Host "  LIVA Banking Harness -- ngrok Remote Access Tunnel Controller" -ForegroundColor White
    Write-Host "  Target Service: Treasury and Reconciliation Workbench" -ForegroundColor White
    Write-Host "  Domain: $Domain" -ForegroundColor Yellow
    Write-Host "  Upstream Port: $Port | Config: $ConfigFile" -ForegroundColor Gray
    Write-Host "==========================================================================" -ForegroundColor Cyan
    Write-Host ""
}

function Test-NgrokBinary {
    $cmd = Get-Command ngrok -ErrorAction SilentlyContinue
    if (-not $cmd) {
        Write-Host "[FAIL] ngrok executable was not found in PATH." -ForegroundColor Red
        Write-Host "       Please install ngrok or add it to system PATH." -ForegroundColor Yellow
        return $false
    }
    $version = (ngrok version 2>&1).Trim()
    Write-Host "[OK] Detected ngrok binary: $version" -ForegroundColor Green
    return $true
}

function Test-RamGuard {
    if ($SkipRamCheck) {
        Write-Host "[WARN] RAM guard check skipped by user request (-SkipRamCheck)." -ForegroundColor Yellow
        return $true
    }

    if (Test-Path $RamGuardScript) {
        Write-Host "[INFO] Executing RAM pre-flight guardrail (>= 4.0GB free, <= 80% load)..." -ForegroundColor Cyan
        & powershell -ExecutionPolicy Bypass -File $RamGuardScript
        if ($LASTEXITCODE -ne 0) {
            Write-Host "[FAIL] RAM Guardrail triggered: System memory is insufficient to safely launch tunnel." -ForegroundColor Red
            return $false
        }
    } else {
        Write-Host "[WARN] ram-guard.ps1 not found at $RamGuardScript. Performing inline memory check..." -ForegroundColor Yellow
        $os = Get-CimInstance Win32_OperatingSystem
        $freeGB = [math]::Round($os.FreePhysicalMemory / 1MB, 2)
        if ($freeGB -lt 4.0) {
            Write-Host "[FAIL] Free RAM ($freeGB GB) is below 4.0GB safety threshold." -ForegroundColor Red
            return $false
        }
        Write-Host "[OK] Free RAM: $freeGB GB (Healthy)." -ForegroundColor Green
    }
    return $true
}

function Test-UpstreamPort {
    param([int]$CheckPort)
    Write-Host "[INFO] Testing upstream Treasury Workbench service at http://127.0.0.1:$CheckPort ..." -ForegroundColor Cyan
    try {
        $tcp = New-Object System.Net.Sockets.TcpClient
        $asyncResult = $tcp.BeginConnect("127.0.0.1", $CheckPort, $null, $null)
        $wait = $asyncResult.AsyncWaitHandle.WaitOne(1000, $false)
        if ($wait -and $tcp.Connected) {
            $tcp.EndConnect($asyncResult)
            $tcp.Close()
            Write-Host "[OK] Treasury Workbench is actively listening on port $CheckPort." -ForegroundColor Green
            return $true
        } else {
            $tcp.Close()
            Write-Host "[WARN] Port $CheckPort is NOT currently listening." -ForegroundColor Yellow
            Write-Host "       (Ensure Treasury Workbench is running via 'npm run dev' or standalone web server before serving traffic)" -ForegroundColor DarkYellow
            return $false
        }
    } catch {
        Write-Host "[WARN] Could not connect to port ${CheckPort}: $_" -ForegroundColor Yellow
        return $false
    }
}

function Test-Configuration {
    if (-not (Test-Path $ResolvedConfigFile)) {
        Write-Host "[FAIL] Configuration file does not exist: $ResolvedConfigFile" -ForegroundColor Red
        return $false
    }
    Write-Host "[INFO] Validating ngrok configuration syntax: $ResolvedConfigFile ..." -ForegroundColor Cyan
    $checkOutput = (ngrok config check --config $ResolvedConfigFile 2>&1)
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[FAIL] Configuration validation failed:" -ForegroundColor Red
        Write-Host $checkOutput -ForegroundColor Red
        return $false
    }
    Write-Host "[OK] $checkOutput" -ForegroundColor Green
    return $true
}

function Get-TunnelStatus {
    $inspectApi = "http://127.0.0.1:4040/api/tunnels"
    try {
        $response = Invoke-RestMethod -Uri $inspectApi -Method Get -TimeoutSec 3 -ErrorAction Stop
        return $response
    } catch {
        return $null
    }
}

function Start-Tunnel {
    Show-Header

    if (-not (Test-NgrokBinary)) { exit 1 }
    if (-not (Test-RamGuard)) { exit 1 }
    if (-not (Test-Configuration)) { exit 1 }
    $null = Test-UpstreamPort -CheckPort $Port

    # Check if already running
    $existing = Get-TunnelStatus
    if ($existing -and $existing.tunnels -and $existing.tunnels.Count -gt 0) {
        Write-Host ""
        Write-Host "[WARN] ngrok tunnel is ALREADY running!" -ForegroundColor Yellow
        $existing.tunnels | ForEach-Object {
            Write-Host "       Name: $($_.name) | Public URL: $($_.public_url) -> $($_.config.addr)" -ForegroundColor Green
        }
        Write-Host "       Local Inspector: http://127.0.0.1:4040" -ForegroundColor Cyan
        return
    }

    if (-not (Test-Path $DeployDir)) {
        New-Item -ItemType Directory -Path $DeployDir -Force | Out-Null
    }

    $ngrokArgs = if ($FallbackEphemeral) {
        "http $Port"
    } else {
        "start treasury-workbench --config `"$ResolvedConfigFile`""
    }

    if ($Foreground) {
        Write-Host "[INFO] Starting ngrok tunnel in foreground (Ctrl+C to stop)..." -ForegroundColor Cyan
        & ngrok.exe ($ngrokArgs -split ' ')
    } else {
        Write-Host "[INFO] Starting ngrok tunnel in background process..." -ForegroundColor Cyan
        
        $pinfo = New-Object System.Diagnostics.ProcessStartInfo
        $pinfo.FileName = "ngrok.exe"
        $pinfo.Arguments = "$ngrokArgs --log `"$LogFile`""
        $pinfo.UseShellExecute = $false
        $pinfo.CreateNoWindow = $true
        $pinfo.WorkingDirectory = $ProjectRoot

        $proc = [System.Diagnostics.Process]::Start($pinfo)
        $proc.Id | Set-Content -Path $PidFile -Force

        Write-Host "[INFO] Spawned background process PID: $($proc.Id). Waiting for tunnel initialization..." -ForegroundColor Cyan
        
        # Wait up to 6 seconds for endpoint initialization
        $ready = $false
        for ($i = 0; $i -lt 12; $i++) {
            Start-Sleep -Milliseconds 500
            $status = Get-TunnelStatus
            if ($status -and $status.tunnels -and $status.tunnels.Count -gt 0) {
                $ready = $true
                break
            }
        }

        if ($ready) {
            Write-Host ""
            Write-Host "==========================================================================" -ForegroundColor Green
            Write-Host "  SUCCESS: ngrok Remote Access Tunnel Active" -ForegroundColor Green
            Write-Host "==========================================================================" -ForegroundColor Green
            $status.tunnels | ForEach-Object {
                Write-Host "  * Tunnel Name : $($_.name)" -ForegroundColor White
                Write-Host "  * Public URL  : $($_.public_url)" -ForegroundColor Yellow
                Write-Host "  * Upstream    : $($_.config.addr)" -ForegroundColor White
                Write-Host "  * Protocol    : $($_.proto)" -ForegroundColor White
            }
            Write-Host "  * Web Inspector: http://127.0.0.1:4040" -ForegroundColor Cyan
            Write-Host "  * PID File    : $PidFile (PID: $($proc.Id))" -ForegroundColor Gray
            Write-Host "  * Log File    : $LogFile" -ForegroundColor Gray
            Write-Host "==========================================================================" -ForegroundColor Green
        } else {
            Write-Host "[WARN] Tunnel process started (PID: $($proc.Id)), but API did not respond within 6s." -ForegroundColor Yellow
            Write-Host "       Check log file for error diagnostics: $LogFile" -ForegroundColor Yellow
            if (Test-Path $LogFile) {
                $logContent = Get-Content $LogFile -Tail 20
                $logContent | ForEach-Object { Write-Host "       $_" -ForegroundColor DarkYellow }
                if ($logContent -match "ERR_NGROK_313|ERR_NGROK_314") {
                    Write-Host ""
                    Write-Host "[DIAGNOSTIC] The configured domain resource 'rd_3JHcwHJK1iUTdwx1D22XRMeUA5K' requires a paid plan or static domain activation on ngrok dashboard." -ForegroundColor Yellow
                    Write-Host "             To launch an immediate ephemeral remote access tunnel for testing, run:" -ForegroundColor Cyan
                    Write-Host "             .\scripts\deploy-ngrok-tunnel.ps1 -Action start -FallbackEphemeral" -ForegroundColor White
                }
            }
        }
    }
}

function Show-Status {
    Show-Header
    $status = Get-TunnelStatus
    if ($status -and $status.tunnels -and $status.tunnels.Count -gt 0) {
        Write-Host "[STATUS: ACTIVE] ngrok tunnel is RUNNING." -ForegroundColor Green
        Write-Host ""
        $status.tunnels | ForEach-Object {
            Write-Host "  * Tunnel Name      : $($_.name)" -ForegroundColor White
            Write-Host "  * Public URL       : $($_.public_url)" -ForegroundColor Yellow
            Write-Host "  * Local Upstream   : $($_.config.addr)" -ForegroundColor White
            Write-Host "  * Protocol         : $($_.proto)" -ForegroundColor White
            if ($_.metrics) {
                Write-Host "  * HTTP Requests    : Total=$($_.metrics.http.count), Rate1m=$($_.metrics.http.rate1)" -ForegroundColor Gray
            }
        }
        Write-Host "  * Web Inspector    : http://127.0.0.1:4040" -ForegroundColor Cyan
        if (Test-Path $PidFile) {
            $savedPid = (Get-Content $PidFile -Raw).Trim()
            Write-Host "  * Tracking PID     : $savedPid" -ForegroundColor Gray
        }
    } else {
        Write-Host "[STATUS: INACTIVE] No active ngrok tunnel detected." -ForegroundColor Yellow
        if (Test-Path $PidFile) {
            $savedPid = (Get-Content $PidFile -Raw).Trim()
            Write-Host "  * Orphan PID File  : $savedPid (Process may have terminated)" -ForegroundColor DarkYellow
        }
    }
}

function Stop-Tunnel {
    Show-Header
    Write-Host "[INFO] Stopping ngrok tunnel..." -ForegroundColor Cyan

    $stopped = $false

    # Attempt to kill by saved PID
    if (Test-Path $PidFile) {
        $savedPid = (Get-Content $PidFile -Raw).Trim()
        try {
            $targetProc = Get-Process -Id $savedPid -ErrorAction SilentlyContinue
            if ($targetProc -and $targetProc.ProcessName -match "ngrok") {
                Stop-Process -Id $savedPid -Force
                Write-Host "[OK] Terminated ngrok process PID $savedPid." -ForegroundColor Green
                $stopped = $true
            }
        } catch {}
        Remove-Item -Path $PidFile -Force -ErrorAction SilentlyContinue
    }

    # Also terminate any remaining ngrok.exe processes if still alive
    $procs = Get-Process -Name ngrok -ErrorAction SilentlyContinue
    if ($procs) {
        foreach ($p in $procs) {
            Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
            Write-Host "[OK] Terminated ngrok instance PID $($p.Id)." -ForegroundColor Green
            $stopped = $true
        }
    }

    if (-not $stopped) {
        Write-Host "[INFO] No running ngrok processes found." -ForegroundColor Gray
    } else {
        Write-Host "[SUCCESS] All ngrok tunnel instances stopped." -ForegroundColor Green
    }
}

function Inspect-Tunnel {
    Show-Header
    $status = Get-TunnelStatus
    if (-not $status) {
        Write-Host "[WARN] ngrok is not running. Launching status check..." -ForegroundColor Yellow
        Show-Status
        return
    }

    $inspectorUrl = "http://127.0.0.1:4040"
    Write-Host "[INFO] Opening ngrok Web Inspector: $inspectorUrl ..." -ForegroundColor Cyan
    try {
        Start-Process $inspectorUrl
    } catch {
        Write-Host "[INFO] Point your web browser to $inspectorUrl to inspect traffic." -ForegroundColor Yellow
    }
}

# Main Dispatcher
switch ($Action.ToLower()) {
    'test' {
        Show-Header
        Write-Host "[TEST PHASE 1/4] Checking ngrok binary..." -ForegroundColor Cyan
        $binOk = Test-NgrokBinary
        Write-Host ""
        Write-Host "[TEST PHASE 2/4] Checking RAM guardrails..." -ForegroundColor Cyan
        $ramOk = Test-RamGuard
        Write-Host ""
        Write-Host "[TEST PHASE 3/4] Validating YAML configuration syntax..." -ForegroundColor Cyan
        $cfgOk = Test-Configuration
        Write-Host ""
        Write-Host "[TEST PHASE 4/4] Checking upstream Treasury Workbench port..." -ForegroundColor Cyan
        $portOk = Test-UpstreamPort -CheckPort $Port
        Write-Host ""
        if ($binOk -and $ramOk -and $cfgOk) {
            Write-Host "==========================================================================" -ForegroundColor Green
            Write-Host "  PRE-FLIGHT VALIDATION PASSED -- READY TO DEPLOY TUNNEL" -ForegroundColor Green
            Write-Host "  Run: .\scripts\deploy-ngrok-tunnel.ps1 -Action start" -ForegroundColor White
            Write-Host "==========================================================================" -ForegroundColor Green
        } else {
            Write-Host "==========================================================================" -ForegroundColor Red
            Write-Host "  PRE-FLIGHT VALIDATION FAILED -- PLEASE FIX ERRORS ABOVE" -ForegroundColor Red
            Write-Host "==========================================================================" -ForegroundColor Red
            exit 1
        }
    }
    'start' {
        Start-Tunnel
    }
    'status' {
        Show-Status
    }
    'verify' {
        Show-Status
    }
    'inspect' {
        Inspect-Tunnel
    }
    'stop' {
        Stop-Tunnel
    }
    'help' {
        Get-Help $MyInvocation.MyCommand.Path -Full
    }
}
