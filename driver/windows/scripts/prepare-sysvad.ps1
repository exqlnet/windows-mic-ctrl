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

$zipPath = Join-Path $sourceRoot "windows-driver-samples.zip"
$extractRoot = Join-Path $sourceRoot "windows-driver-samples"
$sysvadTarget = Join-Path $sourceRoot "sysvad"

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

Write-Host "已准备 SysVAD 源码: $sysvadTarget"
Write-Host "下一步: 根据 docs/porting-checklist.md 完成品牌名、设备名与接口修改。"
