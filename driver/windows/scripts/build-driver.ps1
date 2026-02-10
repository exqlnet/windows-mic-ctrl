Param(
  [string]$SysvadRoot = "..\src\upstream\sysvad",
  [string]$Configuration = "Release",
  [string]$Platform = "x64",
  [string]$OutputRoot = "..\artifacts\driver",
  [switch]$DisableInfVerification = $true
)

$ErrorActionPreference = "Stop"

function Resolve-VSWhere {
  $path = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
  if (Test-Path $path) { return $path }
  return $null
}

function Resolve-MsBuild {
  $cmd = Get-Command msbuild -ErrorAction SilentlyContinue
  if ($cmd) { return $cmd.Source }

  $vswhere = Resolve-VSWhere
  if ($vswhere) {
    $installPath = & $vswhere -latest -products * -requires Microsoft.Component.MSBuild -property installationPath
    if ($installPath) {
      $candidate = Join-Path $installPath "MSBuild\Current\Bin\MSBuild.exe"
      if (Test-Path $candidate) { return $candidate }
    }
  }

  return $null
}

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$sysvadPath = Resolve-Path (Join-Path $scriptDir $SysvadRoot) -ErrorAction SilentlyContinue
if (-not $sysvadPath) {
  throw "未找到 SysVAD 源码目录，请先运行 prepare-sysvad.ps1。"
}
$sysvadPath = $sysvadPath.Path

$msbuildPath = Resolve-MsBuild
if (-not $msbuildPath) {
  throw "未检测到 msbuild，请安装 Visual Studio Build Tools + WDK。"
}

$solution = Get-ChildItem -Path $sysvadPath -Filter *.sln -Recurse | Select-Object -First 1
if (-not $solution) {
  throw "未找到 .sln，请确认 SysVAD 源码完整。"
}

$msbuildArgs = @(
  $solution.FullName,
  "/m",
  "/p:Configuration=$Configuration",
  "/p:Platform=$Platform"
)

if ($DisableInfVerification) {
  $msbuildArgs += "/p:EnableInfVerif=false"
  $msbuildArgs += "/p:RunInfVerification=false"
}

Write-Host "使用 MSBuild: $msbuildPath"
Write-Host "使用解决方案: $($solution.FullName)"
Write-Host "构建参数: $($msbuildArgs -join ' ')"
& $msbuildPath @msbuildArgs
$msbuildExitCode = $LASTEXITCODE
if ($msbuildExitCode -ne 0) {
  Write-Warning "msbuild 返回非零退出码: $msbuildExitCode，将尝试收集已生成产物。"
}

$outputPath = Join-Path $scriptDir $OutputRoot
if (Test-Path $outputPath) {
  Remove-Item -Recurse -Force $outputPath
}
New-Item -ItemType Directory -Path $outputPath -Force | Out-Null

$artifacts = Get-ChildItem -Path $sysvadPath -Recurse -File | Where-Object {
  $_.Extension -in ".sys", ".cat", ".inf"
}

if (-not $artifacts) {
  throw "未找到驱动产物（.sys/.cat/.inf），请检查工程配置。"
}

$artifacts | ForEach-Object {
  Copy-Item $_.FullName -Destination (Join-Path $outputPath $_.Name) -Force
}

Write-Host "驱动构建产物已输出到: $outputPath"
Get-ChildItem -Path $outputPath -File | Format-Table Name, Length -AutoSize

if ($msbuildExitCode -ne 0) {
  throw "msbuild 存在失败项（退出码: $msbuildExitCode），但已收集到部分产物，请人工确认可用性。"
}
