#Requires -Version 7.0
<#
.SYNOPSIS
  Yayin oncesi yardimci: surumu Cargo.toml'den okur ve tag yolunu dogrular.
.DESCRIPTION
  Surumu once package/Cargo.toml, yoksa kok Cargo.toml icinden okur.
  -Tag verilirse "v<surum>" ile eslesmesini ve refs/tags/<tag> ref'ini kontrol eder.
.EXAMPLE
  .\scripts\yayin.ps1
  .\scripts\yayin.ps1 -Tag v5.0.0
#>
[CmdletBinding()]
param(
    [string]$Tag
)

$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$manifest = Join-Path $root "package\Cargo.toml"
if (-not (Test-Path $manifest)) {
    $manifest = Join-Path $root "Cargo.toml"
}
if (-not (Test-Path $manifest)) {
    throw "Cargo.toml bulunamadi: $manifest"
}

$icerik = Get-Content -Path $manifest -Raw
$match = [regex]::Match($icerik, '(?m)^[ \t]*version[ \t]*=[ \t]*"([^"]+)"')
$surum = $match.Groups[1].Value
if (-not $surum) {
    throw "Surum okunamadi: $manifest"
}

$beklenenTag = "v$surum"
Write-Host "Surum: $surum  (kaynak: $manifest)"
Write-Host "Beklenen tag: $beklenenTag"

if (-not $Tag) {
    $mevcut = $null
    try { $mevcut = (git -C $root describe --tags --exact-match HEAD 2>$null).Trim() } catch { }
    if ($mevcut) {
        Write-Host "Mevcut HEAD tag'i: $mevcut"
        $Tag = $mevcut
    } else {
        Write-Host "Kullanim: .\scripts\yayin.ps1 -Tag $beklenenTag"
        return
    }
}

if ($Tag -ne $beklenenTag) {
    throw "Tag '$Tag', beklenen '$beklenenTag' ile uyumsuz (Cargo.toml surumu: $surum)."
}

$ref = git -C $root rev-parse -q --verify "refs/tags/$Tag" 2>$null
if (-not $ref) {
    Write-Warning "Tag ref'i yerel olarak bulunamadi: refs/tags/$Tag (henuz olusturulmamis olabilir)."
} else {
    Write-Host "Tag ref'i dogrulandi: $ref"
}

Write-Host "Tag $Tag surum $surum ile uyumlu. Yayina hazir."
