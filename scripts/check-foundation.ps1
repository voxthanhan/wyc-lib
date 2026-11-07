#Requires -Version 7.0
[CmdletBinding()]
param(
    [string]$EvidenceDirectory = 'ci-evidence',
    [string]$ExpectedCommit = '',
    [switch]$Offline,
    [switch]$RequireClean
)

$ErrorActionPreference = 'Stop'
$env:CARGO_INCREMENTAL = '0'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
Push-Location $repoRoot
try {
    $commit = (& git rev-parse --verify HEAD).Trim()
    if ($LASTEXITCODE -ne 0) { throw 'A committed Git checkout is required.' }
    if ($ExpectedCommit -and $commit -ne $ExpectedCommit) { throw 'Checkout does not match ExpectedCommit.' }
    $tree = (& git rev-parse 'HEAD^{tree}').Trim()
    $before = @(& git status --porcelain --untracked-files=normal)
    if ($LASTEXITCODE -ne 0) { throw 'Cannot inspect checkout cleanliness.' }
    if ($RequireClean -and $before.Count) { throw 'Checkout must be clean before verification.' }
    New-Item -ItemType Directory -Force -Path $EvidenceDirectory | Out-Null
    $evidencePath = (Resolve-Path $EvidenceDirectory).Path
    $checks = [System.Collections.Generic.List[object]]::new()
    $runUrl = $null
    if ($env:GITHUB_ACTIONS -eq 'true') {
        $runUrl = "$env:GITHUB_SERVER_URL/$env:GITHUB_REPOSITORY/actions/runs/$env:GITHUB_RUN_ID"
    }
    $summary = [ordered]@{
        evidence_version = '1'
        commit = $commit
        tree = $tree
        started_at_utc = [DateTimeOffset]::UtcNow.ToString('o')
        platform = [System.Runtime.InteropServices.RuntimeInformation]::OSDescription
        github_run_url = $runUrl
        github_run_attempt = $env:GITHUB_RUN_ATTEMPT
        clean_before = ($before.Count -eq 0)
        clean_after = $false
        offline = [bool]$Offline
        checks = $checks
        passed = $false
    }

    function Invoke-Check([string]$Name, [string]$Command, [string[]]$Arguments) {
        Write-Host "CHECK: $Command $($Arguments -join ' ')"
        $output = & $Command @Arguments 2>&1
        $code = $LASTEXITCODE
        $output | Tee-Object -FilePath (Join-Path $evidencePath "$Name.txt")
        $checks.Add([ordered]@{ name = $Name; command = $Command; arguments = $Arguments; exit_code = $code })
        if ($code -ne 0) { throw "$Name failed with exit code $code" }
    }

    try {
        Invoke-Check 'rustc' 'rustc' @('--version', '--verbose')
        Invoke-Check 'cargo' 'cargo' @('--version')
        Invoke-Check 'python' 'python' @('--version')
        Invoke-Check 'format' 'cargo' @('fmt', '--all', '--', '--check')
        $locked = @('--workspace', '--locked')
        if ($Offline) { $locked += '--offline' }
        Invoke-Check 'clippy' 'cargo' (@('clippy') + $locked + @('--all-targets', '--', '-D', 'warnings'))
        Invoke-Check 'build' 'cargo' (@('build') + $locked)
        Invoke-Check 'test' 'cargo' (@('test') + $locked)
        Invoke-Check 'oracle' 'python' @('scripts/verify_oracles.py')
        $metadataArgs = @('metadata', '--format-version', '1', '--no-deps', '--locked')
        if ($Offline) { $metadataArgs += '--offline' }
        $metadata = & cargo @metadataArgs
        if ($LASTEXITCODE -ne 0) { throw 'cargo metadata failed' }
        $metadataFile = Join-Path $evidencePath 'workspace-metadata.json'
        $metadata | Set-Content -LiteralPath $metadataFile -Encoding utf8
        Invoke-Check 'workspace' 'python' @('scripts/verify_workspace.py', $metadataFile)
        $after = @(& git status --porcelain --untracked-files=normal)
        if ($LASTEXITCODE -ne 0) { throw 'Cannot inspect checkout after verification.' }
        $summary.clean_after = ($after.Count -eq 0)
        if ($RequireClean -and $after.Count) { throw 'Verification changed the checkout.' }
        $summary.passed = $true
    }
    finally {
        $summary['finished_at_utc'] = [DateTimeOffset]::UtcNow.ToString('o')
        $summary | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $evidencePath 'summary.json') -Encoding utf8
    }
    Write-Host "PASS: foundation verified at $commit; evidence: $evidencePath"
}
finally {
    Pop-Location
}
