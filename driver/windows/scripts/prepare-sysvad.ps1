Param(
  [string]$SourceRoot = "..\src\upstream",
  [string]$Branch = "main"
)

$ErrorActionPreference = "Stop"

Write-Host "[Windows Mic Ctrl] 准备 SysVAD 源码"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$rawSourcePath = Join-Path $scriptDir $SourceRoot
if (-not (Test-Path $rawSourcePath)) {
  New-Item -ItemType Directory -Path $rawSourcePath -Force | Out-Null
}
$sourceRoot = (Resolve-Path $rawSourcePath).Path
$sourceBase = Split-Path -Parent $sourceRoot

$zipPath = Join-Path $sourceRoot "windows-driver-samples.zip"
$extractRoot = Join-Path $sourceRoot "windows-driver-samples"
$sysvadTarget = Join-Path $sourceRoot "sysvad"
$wilLegacyTarget = Join-Path $sourceRoot "wil"
$wilTarget = Join-Path $sourceBase "wil"

$downloadUrl = "https://github.com/microsoft/Windows-driver-samples/archive/refs/heads/$Branch.zip"
Write-Host "下载: $downloadUrl"
Invoke-WebRequest -Uri $downloadUrl -OutFile $zipPath

if (Test-Path $extractRoot) {
  Remove-Item -Recurse -Force $extractRoot
}
Expand-Archive -Path $zipPath -DestinationPath $extractRoot -Force

$repoDir = Get-ChildItem -Path $extractRoot -Directory | Select-Object -First 1
if (-not $repoDir) {
  throw "解压后未找到 Windows-driver-samples 目录"
}

$sysvadSource = Join-Path $repoDir.FullName "audio\sysvad"
if (-not (Test-Path $sysvadSource)) {
  throw "未找到 SysVAD 目录: $sysvadSource"
}

if (Test-Path $sysvadTarget) {
  Remove-Item -Recurse -Force $sysvadTarget
}
Copy-Item -Path $sysvadSource -Destination $sysvadTarget -Recurse -Force

$wilSource = Join-Path $repoDir.FullName "wil"
if (Test-Path $wilSource) {
  if (Test-Path $wilLegacyTarget) {
    Remove-Item -Recurse -Force $wilLegacyTarget
  }
  if (Test-Path $wilTarget) {
    Remove-Item -Recurse -Force $wilTarget
  }

  Copy-Item -Path $wilSource -Destination $wilLegacyTarget -Recurse -Force
  Copy-Item -Path $wilSource -Destination $wilTarget -Recurse -Force
  Write-Host "已复制 WIL 目录（兼容路径）: $wilLegacyTarget"
  Write-Host "已复制 WIL 目录（APO 默认路径）: $wilTarget"
}
else {
  Write-Warning "未在上游仓库中找到 wil 目录，某些 APO 项目可能构建失败。"
}

Write-Host "已准备 SysVAD 源码: $sysvadTarget"
Write-Host "下一步: 根据 docs/porting-checklist.md 完成品牌名、设备名与接口修改。"
