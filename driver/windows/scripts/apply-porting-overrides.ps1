Param(
  [string]$SysvadRoot = "..\src\upstream\sysvad",
  [string]$VendorName = "exqlnet",
  [string]$DeviceName = "Windows Mic Ctrl Virtual Mic",
  [string]$ServiceName = "windows_mic_ctrl_virtual_mic",
  [string]$HardwareId = "ROOT\\WINDOWS_MIC_CTRL_VIRTUAL_MIC"
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$sysvadPath = Resolve-Path (Join-Path $scriptDir $SysvadRoot) -ErrorAction SilentlyContinue
if (-not $sysvadPath) {
  throw "未找到 SysVAD 目录，请先运行 prepare-sysvad.ps1。"
}
$sysvadPath = $sysvadPath.Path

$overrideDir = Join-Path $sysvadPath "windows-mic-ctrl-overrides"
if (-not (Test-Path $overrideDir)) {
  New-Item -ItemType Directory -Path $overrideDir -Force | Out-Null
}

$manifest = @{
  vendor_name = $VendorName
  device_name = $DeviceName
  service_name = $ServiceName
  hardware_id = $HardwareId
  generated_at = (Get-Date).ToString("s")
  notes = @(
    "该文件记录 SysVAD 派生品牌化参数",
    "请据此修改驱动源码中的设备名、服务名、端点描述与硬件 ID"
  )
}

$manifestPath = Join-Path $overrideDir "porting-manifest.json"
$manifest | ConvertTo-Json -Depth 5 | Set-Content -Path $manifestPath -Encoding UTF8

Write-Host "已生成派生参数清单: $manifestPath"
Write-Host "下一步：按 docs/porting-checklist.md 修改 SysVAD 代码并重新构建。"
