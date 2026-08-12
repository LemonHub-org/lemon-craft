# cargo-watch.ps1 — run cargo in the background and poll it separately.
#
# WHY: long compiles (this workspace takes 5-25+ minutes for a cold check)
# exceed interactive shell timeouts. Killing the foreground shell leaves
# orphan cargo/rustc processes holding the build lock. This script splits
# the operation in two phases so a shell call can never time out:
#
#   .\tools\cargo-watch.ps1 -Launch -Name wasm-voxygen -Args @("check", "--target", "wasm32-unknown-unknown", "-p", "lemoncraft-voxygen", "--no-default-features", "--features", "wasm-singleplayer")
#   .\tools\cargo-watch.ps1 -Wait -Name wasm-voxygen          # poll until done
#
# Logs land in $LogDir\<Name>.log (stdout) and <Name>.err (stderr).
# `-Wait` exits 0 when the job succeeded, 1 when it failed, 2 when it is
# still running after the deadline.

param(
    [Parameter(Mandatory = $true)][ValidateSet("Launch", "Wait")][string]$Mode,
    [Parameter(Mandatory = $true)][string]$Name,
    [string[]]$Args,
    [int]$WaitSeconds = 7200
)

$LogDir = Join-Path $env:TEMP "opencode"
New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
$StdoutLog = Join-Path $LogDir "$Name.log"
$StderrLog = Join-Path $LogDir "$Name.err"
$PidFile = Join-Path $LogDir "$Name.pid"

function Get-RunningPid {
    if (-not (Test-Path $PidFile)) { return $null }
    $procId = [int](Get-Content $PidFile -Raw).Trim()
    $proc = Get-Process -Id $procId -ErrorAction SilentlyContinue
    if ($proc -and $proc.ProcessName -match 'cargo') { return $procId }
    return $null
}

switch ($Mode) {
    "Launch" {
        if (Get-RunningPid) {
            Write-Output "ALREADY RUNNING (pid $(Get-RunningPid))"
            exit 0
        }
        # A previous compile may have left cargo/rustc orphans holding the lock.
        $deadline = (Get-Date).AddMinutes(10)
        while ((Get-Process -Name cargo, rustc -ErrorAction SilentlyContinue) -and (Get-Date) -lt $deadline) {
            Start-Sleep -Seconds 5
        }
        Remove-Item $StdoutLog, $StderrLog, $PidFile -ErrorAction SilentlyContinue
        $p = Start-Process -FilePath "cargo" -ArgumentList $Args -WorkingDirectory (Get-Location) `
            -WindowStyle Hidden -RedirectStandardOutput $StdoutLog -RedirectStandardError $StderrLog `
            -PassThru
        Set-Content -Path $PidFile -Value $p.Id
        Write-Output "LAUNCHED pid=$($p.Id) log=$StdoutLog"
    }
    "Wait" {
        $deadline = (Get-Date).AddSeconds($WaitSeconds)
        $procId = Get-RunningPid
        while ($procId -and (Get-Date) -lt $deadline) {
            Start-Sleep -Seconds 10
            $procId = Get-RunningPid
        }
        if ($procId) {
            Write-Output "STILL RUNNING pid=$procId (check $StdoutLog)"
            exit 2
        }
        Remove-Item $PidFile -ErrorAction SilentlyContinue
        $exitCode = 0
        if (Test-Path $StderrLog) {
            $content = Get-Content $StderrLog -ErrorAction SilentlyContinue
            $errors = $content | Select-String -Pattern 'could not compile|error\[[0-9A-Z]+\]'
            if ($errors) {
                $exitCode = 1
                Write-Output "--- errors ---"
                $errors | ForEach-Object { $_.Line } | Select-Object -First 15
            }
            if ($exitCode -eq 1) {
                Write-Output "--- stderr tail ---"
                $content | Select-Object -Last 6
            }
        }
        Write-Output "--- last lines ---"
        if (Test-Path $StdoutLog) { Get-Content $StdoutLog -Tail 5 }
        if (Test-Path $StderrLog) { Get-Content $StderrLog -Tail 5 }
        Write-Output "DONE exit=$exitCode"
        exit $exitCode
    }
}
