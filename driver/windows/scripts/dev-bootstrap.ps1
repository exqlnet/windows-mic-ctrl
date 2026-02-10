Param(
  [string]$SourceRoot = "..\src\upstream",
  [string]$Branch = "main",
  [switch]$SkipPrepare
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

function Run-Step {
  Param(
    [string]$Name,
    [string]$ScriptPath,
    [hashtable]$Args = @{}
  )

  Write-Host ""
  Write-Host "=== $Name ==="
  & $ScriptPath @Args
}

$checkScript = Join-Path $scriptDir "check-toolchain.ps1"
$prepareScript = Join-Path $scriptDir "prepare-sysvad.ps1"
$overrideScript = Join-Path $scriptDir "apply-porting-overrides.ps1"

Run-Step -Name "检查工具链" -ScriptPath $checkScript

if (-not $SkipPrepare) {
  Run-Step -Name "准备 SysVAD 源码" -ScriptPath $prepareScript -Args @{
    SourceRoot = $SourceRoot
    Branch = $Branch
  }
}

Run-Step -Name "生成派生参数清单" -ScriptPath $overrideScript -Args @{
  SysvadRoot = "..\src\upstream\sysvad"
}

Write-Host ""
Write-Host "[Windows Mic Ctrl] 开发引导完成。"
Write-Host "下一步："
Write-Host "1) 按 driver/windows/docs/porting-checklist.md 完成 SysVAD 派生改造"
Write-Host "2) 执行 driver/windows/scripts/build-driver.ps1"
Write-Host "3) 管理员运行 driver/windows/scripts/install-driver-test.ps1"
