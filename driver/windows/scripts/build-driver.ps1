Param(
  [string]$SysvadRoot = "..\src\upstream\sysvad",
  [string]$Configuration = "Release",
  [string]$Platform = "x64",
  [string]$OutputRoot = "..\artifacts\driver",
  [switch]$DisableInfVerification = $true,
  [string]$TargetInfName = "windows-mic-ctrl-virtual-mic.inf",
  [string]$TargetSysName = "windows-mic-ctrl-virtual-mic.sys"
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

function Resolve-Inf2Cat {
  $cmd = Get-Command inf2cat -ErrorAction SilentlyContinue
  if ($cmd) { return $cmd.Source }

  $kitsRoot = Join-Path ${env:ProgramFiles(x86)} "Windows Kits\10\bin"
  if (-not (Test-Path $kitsRoot)) { return $null }

  $found = Get-ChildItem -Path $kitsRoot -Recurse -File -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -ieq "Inf2Cat.exe" } |
    Sort-Object FullName -Descending |
    Select-Object -First 1

  if ($found) { return $found.FullName }
  return $null
}

function Get-PreferredInf {
  Param(
    [string]$SysvadPath,
    [string]$ScriptDir,
    [string]$NameHint
  )

  $candidates = @(
    (Join-Path $ScriptDir "..\inf\$NameHint"),
    (Join-Path $SysvadPath "TabletAudioSample\x64\Release\ComponentizedAudioSample.inf"),
    (Join-Path $SysvadPath "TabletAudioSample\x64\Release\ComponentizedApoSample.inf"),
    (Join-Path $SysvadPath "TabletAudioSample\x64\Release\ComponentizedAudioSampleExtension.inf")
  )

  foreach ($candidate in $candidates) {
    $resolved = Resolve-Path $candidate -ErrorAction SilentlyContinue
    if ($resolved) {
      return $resolved.Path
    }
  }

  $allInf = Get-ChildItem -Path $SysvadPath -Recurse -File -Filter *.inf -ErrorAction SilentlyContinue |
    Sort-Object LastWriteTime -Descending
  if ($allInf.Count -gt 0) {
    return $allInf[0].FullName
  }

  return $null
}

function Get-PreferredSys {
  Param([string]$SysvadPath)

  $preferred = @(
    (Join-Path $SysvadPath "TabletAudioSample\x64\Release\TabletAudioSample.sys")
  )

  foreach ($candidate in $preferred) {
    $resolved = Resolve-Path $candidate -ErrorAction SilentlyContinue
    if ($resolved) {
      return $resolved.Path
    }
  }

  $allSys = Get-ChildItem -Path $SysvadPath -Recurse -File -Filter *.sys -ErrorAction SilentlyContinue |
    Sort-Object LastWriteTime -Descending

  if ($allSys.Count -gt 0) {
    return $allSys[0].FullName
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

$msbuildExitCode = 0
& $msbuildPath @msbuildArgs
$msbuildExitCode = $LASTEXITCODE
if ($msbuildExitCode -ne 0) {
  Write-Warning "msbuild 返回非零退出码: $msbuildExitCode，将继续尝试收集可用产物。"
}

$outputPath = Join-Path $scriptDir $OutputRoot
if (Test-Path $outputPath) {
  Remove-Item -Recurse -Force $outputPath
}
New-Item -ItemType Directory -Path $outputPath -Force | Out-Null

$preferredInf = Get-PreferredInf -SysvadPath $sysvadPath -ScriptDir $scriptDir -NameHint $TargetInfName
if (-not $preferredInf) {
  throw "未找到可用 INF 文件，无法生成驱动包。"
}

$sysPath = Get-PreferredSys -SysvadPath $sysvadPath
if (-not $sysPath) {
  throw "未找到 .sys 产物，无法生成驱动包。"
}

Copy-Item $preferredInf (Join-Path $outputPath $TargetInfName) -Force
Copy-Item $sysPath (Join-Path $outputPath $TargetSysName) -Force

$inf2cat = Resolve-Inf2Cat
if (-not $inf2cat) {
  throw "未检测到 inf2cat，无法生成 .cat 文件。"
}

Write-Host "使用 INF 生成 catalog: $TargetInfName"
Push-Location $outputPath
try {
  & $inf2cat /driver:$outputPath /os:10_X64
  if ($LASTEXITCODE -ne 0) {
    throw "Inf2Cat 生成失败，退出码: $LASTEXITCODE"
  }
}
finally {
  Pop-Location
}

$hasSys = @(Get-ChildItem -Path $outputPath -File -Filter *.sys -ErrorAction SilentlyContinue).Count -gt 0
$hasInf = @(Get-ChildItem -Path $outputPath -File -Filter *.inf -ErrorAction SilentlyContinue).Count -gt 0
$hasCat = @(Get-ChildItem -Path $outputPath -File -Filter *.cat -ErrorAction SilentlyContinue).Count -gt 0

if (-not ($hasSys -and $hasInf -and $hasCat)) {
  throw "驱动产物不完整（需要 .sys/.inf/.cat）。"
}

Write-Host "驱动构建产物已输出到: $outputPath"
Get-ChildItem -Path $outputPath -File | Format-Table Name, Length -AutoSize

if ($msbuildExitCode -ne 0) {
  Write-Warning "msbuild 存在失败项（退出码: $msbuildExitCode），但已生成完整驱动产物，请人工确认可用性。"
}
