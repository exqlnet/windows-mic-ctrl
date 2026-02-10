Param(
  [string]$InfPath = "..\inf\windows-mic-ctrl-virtual-mic.inf",
  [switch]$EnableTestSigning
)

$ErrorActionPreference = "Stop"

function Assert-Admin {
  $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
  $principal = New-Object Security.Principal.WindowsPrincipal($identity)
  if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw "请使用管理员 PowerShell 执行该脚本。"
  }
}

function Is-TestSigningEnabled {
  $output = bcdedit /enum {current} | Out-String
  return $output.ToLower().Contains("testsigning") -and $output.ToLower().Contains("yes")
}

Assert-Admin

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$resolvedInf = Resolve-Path (Join-Path $scriptDir $InfPath) -ErrorAction SilentlyContinue
if (-not $resolvedInf) {
  throw "INF 文件不存在: $InfPath"
}

if ($EnableTestSigning -and -not (Is-TestSigningEnabled)) {
  Write-Host "正在启用 Test Mode..."
  bcdedit /set testsigning on | Out-Host
  Write-Warning "请重启系统后再次运行安装脚本。"
  exit 0
}

Write-Host "安装驱动 INF: $resolvedInf"
pnputil /add-driver "$resolvedInf" /install | Out-Host

Write-Host "检查驱动服务与录制端点..."
sc query windows_mic_ctrl_virtual_mic | Out-Host
Get-PnpDevice -Class AudioEndpoint | Where-Object { $_.FriendlyName -like "*Windows Mic Ctrl Virtual Mic*" } | Format-List | Out-Host

Write-Host "安装流程完成。若未看到录制端点，请检查驱动签名、重启后再验证。"
