$ErrorActionPreference = "Stop"

function Assert-Admin {
  $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
  $principal = New-Object Security.Principal.WindowsPrincipal($identity)
  if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw "请使用管理员 PowerShell 执行该脚本。"
  }
}

Assert-Admin

Write-Host "停止并删除服务（如果存在）..."
sc stop windows_mic_ctrl_virtual_mic | Out-Null
sc delete windows_mic_ctrl_virtual_mic | Out-Null

Write-Host "移除驱动程序包（需要根据实际 Published Name 调整）..."
pnputil /enum-drivers | Out-Host
Write-Host "请从上方找到 oemXX.inf（Provider 含 exqlnet 或 Device 含 Windows Mic Ctrl），然后执行："
Write-Host "pnputil /delete-driver oemXX.inf /uninstall /force"
