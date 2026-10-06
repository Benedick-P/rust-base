# Compile and run every case under cases/, capture compiler output or runtime panic.
# Usage: pwsh -File capture.ps1
#
# NOTE: intentionally ASCII-only (PowerShell 5.1 reads BOM-less UTF-8 scripts as ANSI
#       and would mangle non-ASCII literals). Chinese labels come from summary.cjs.
#       Native stderr is captured via *file redirection* (2> file), which works
#       consistently across PowerShell 5.1 / 7 -- piping 2>&1 into Out-String does not.
$ErrorActionPreference = 'Continue'
$cases = $PSScriptRoot
$rows = @()

foreach ($dir in Get-ChildItem $cases -Directory | Sort-Object Name) {
    $src = Join-Path $dir.FullName "main.rs"
    if (-not (Test-Path $src)) { continue }

    $exe = Join-Path $dir.FullName "out.exe"
    $diag = Join-Path $dir.FullName "diagnostic.txt"
    $tmpErr = Join-Path $dir.FullName "_build.err"
    Remove-Item $exe, $tmpErr, (Join-Path $dir.FullName "out.pdb") -Force -ErrorAction SilentlyContinue

    # capture compiler stderr into a file, then read it (encoding-safe)
    & rustc --edition 2021 $src -o $exe 2> $tmpErr | Out-Null
    $buildErr = ''
    if (Test-Path $tmpErr) { $buildErr = [System.IO.File]::ReadAllText($tmpErr).Trim() }

    if (-not (Test-Path $exe)) {
        [System.IO.File]::WriteAllText($diag, "=== COMPILE_FAILED ===`n$buildErr", [System.Text.UTF8Encoding]::new($false))
        $code = ([regex]::Match($buildErr, 'error\[(E\d+)\]')).Groups[1].Value
        if (-not $code) { $code = 'no-code' }
        $rows += [pscustomobject]@{ Case = $dir.Name; Kind = 'compile_error'; Code = $code }
        Remove-Item $tmpErr -Force -ErrorAction SilentlyContinue
        continue
    }

    # run it, capture both streams into a file
    $tmpOut = Join-Path $dir.FullName "_run.out"
    Remove-Item $tmpOut -Force -ErrorAction SilentlyContinue
    & $exe > $tmpOut 2>&1
    $exit = $LASTEXITCODE
    $runClean = ''
    if (Test-Path $tmpOut) { $runClean = [System.IO.File]::ReadAllText($tmpOut).Trim() }

    if ($exit -eq 0) {
        [System.IO.File]::WriteAllText($diag, "=== OK exit 0 ===`n$runClean", [System.Text.UTF8Encoding]::new($false))
        $rows += [pscustomobject]@{ Case = $dir.Name; Kind = 'ok'; Code = 'exit 0' }
    }
    else {
        [System.IO.File]::WriteAllText($diag, "=== PANIC exit $exit ===`n$runClean", [System.Text.UTF8Encoding]::new($false))
        $rows += [pscustomobject]@{ Case = $dir.Name; Kind = 'panic'; Code = "exit $exit" }
    }
    Remove-Item $exe, $tmpErr, $tmpOut, (Join-Path $dir.FullName "out.pdb") -Force -ErrorAction SilentlyContinue
}

$out = Join-Path $cases "_raw.tsv"
$rows | ForEach-Object { "$($_.Case)`t$($_.Kind)`t$($_.Code)" } | Set-Content $out -Encoding UTF8
Write-Output "captured $($rows.Count) cases -> $out"
