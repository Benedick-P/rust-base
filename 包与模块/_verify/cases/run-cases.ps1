# 逐个编译 cases 下的最小工程，捕获真实报错
# 用法（需要 PowerShell 7+）: pwsh -File run-cases.ps1
#
# 注意：PS7 把原生命令的 stderr 直接作为 UTF-8 文本返回（无 CategoryInfo / At line: 包装），
#       所以这里不需要像 PS5.1 那样手工清理包装行。
$ErrorActionPreference = 'SilentlyContinue'
$cases = $PSScriptRoot
$env:CARGO_TARGET_DIR = Join-Path $cases "..\target-cases"

if ($PSVersionTable.PSVersion.Major -lt 7) {
    Write-Warning "当前 PowerShell $($PSVersionTable.PSVersion)：stderr 可能被包一层且为 UTF-16，输出会不干净"
}

$results = @()
foreach ($dir in Get-ChildItem $cases -Directory | Sort-Object Name) {
    $name = $dir.Name
    $toml = Join-Path $dir.FullName "Cargo.toml"
    $errFile = Join-Path $dir.FullName "diagnostic.txt"

    if (Test-Path $toml) {
        # cargo 工程：用 cargo check 捕获（模块类报错需要 cargo 的模块解析）
        Push-Location $dir.FullName
        $out = (cargo check --message-format short 2>&1 | Out-String)
        Pop-Location
    }
    else {
        # 单文件：rustc 即可
        $src = Join-Path $dir.FullName "src\main.rs"
        $exe = Join-Path $dir.FullName "out.exe"
        $out = (rustc --edition 2021 --crate-type bin $src -o $exe 2>&1 | Out-String)
        Remove-Item $exe, ($exe -replace '\.exe$', '.pdb') -Force -ErrorAction SilentlyContinue
    }

    # PS7 下 $out 已是干净文本，直接写入（UTF-8 无 BOM）
    $clean = ($out -split "`r?`n" | Where-Object { $_ -notmatch '^\s*$' }) -join "`n"
    [System.IO.File]::WriteAllText($errFile, $clean, [System.Text.UTF8Encoding]::new($false))

    $code = ([regex]::Match($clean, 'error\[(E\d+)\]')).Groups[1].Value
    if (-not $code) { $code = ([regex]::Match($clean, 'error: (.+)')).Groups[1].Value }
    $firstLine = (($clean -split "`n") | Where-Object { $_ -match 'error' } | Select-Object -First 1)
    $results += [pscustomobject]@{ Case = $name; Code = $code; First = $firstLine.Trim() }
}

$results | Format-Table -AutoSize -Wrap
$results | ForEach-Object { "$($_.Case)`t$($_.Code)`t$($_.First)" } |
    Set-Content (Join-Path $cases "_summary.txt") -Encoding UTF8
