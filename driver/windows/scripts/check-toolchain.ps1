$ErrorActionPreference = "Stop"

function Resolve-Tool([string]$Name) {
  $cmd = Get-Command $Name -ErrorAction SilentlyContinue
  if ($cmd) { return $cmd.Source }
  return $null
}

$results = @(
  [PSCustomObject]@{ Tool = "msbuild"; Required = "Build"; Path = (Resolve-Tool "msbuild") },
  [PSCustomObject]@{ Tool = "signtool"; Required = "Sign"; Path = (Resolve-Tool "signtool") },
  [PSCustomObject]@{ Tool = "inf2cat"; Required = "Catalog"; Path = (Resolve-Tool "inf2cat") },
  [PSCustomObject]@{ Tool = "pnputil"; Required = "Install"; Path = (Resolve-Tool "pnputil") },
  [PSCustomObject]@{ Tool = "bcdedit"; Required = "TestMode"; Path = (Resolve-Tool "bcdedit") }
)

$results | ForEach-Object {
  $_ | Add-Member -NotePropertyName Status -NotePropertyValue ($(if ($_.Path) { "OK" } else { "MISSING" }))
}

$results | Format-Table -AutoSize

$missingBuild = $results | Where-Object { $_.Required -in @("Build", "Catalog") -and -not $_.Path }
if ($missingBuild) {
  Write-Error "缺少驱动构建必要工具，请安装 Visual Studio Build Tools + Windows WDK。"
}
