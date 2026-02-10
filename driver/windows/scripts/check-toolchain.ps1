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

function Resolve-KitTool([string]$Name) {
  $cmd = Get-Command $Name -ErrorAction SilentlyContinue
  if ($cmd) { return $cmd.Source }

  $kitsRoot = Join-Path ${env:ProgramFiles(x86)} "Windows Kits\10\bin"
  if (-not (Test-Path $kitsRoot)) { return $null }

  $found = Get-ChildItem -Path $kitsRoot -Recurse -File -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -ieq "$Name.exe" } |
    Sort-Object FullName -Descending |
    Select-Object -First 1

  if ($found) { return $found.FullName }
  return $null
}

$results = @(
  [PSCustomObject]@{ Tool = "msbuild"; Required = "Build"; Path = (Resolve-MsBuild) },
  [PSCustomObject]@{ Tool = "signtool"; Required = "Sign"; Path = (Resolve-KitTool "signtool") },
  [PSCustomObject]@{ Tool = "inf2cat"; Required = "Catalog"; Path = (Resolve-KitTool "inf2cat") },
  [PSCustomObject]@{ Tool = "pnputil"; Required = "Install"; Path = (Resolve-KitTool "pnputil") },
  [PSCustomObject]@{ Tool = "bcdedit"; Required = "TestMode"; Path = (Resolve-KitTool "bcdedit") }
)

$results | ForEach-Object {
  $_ | Add-Member -NotePropertyName Status -NotePropertyValue ($(if ($_.Path) { "OK" } else { "MISSING" }))
}

$results | Format-Table -AutoSize

$missingBuild = $results | Where-Object { $_.Required -in @("Build", "Catalog") -and -not $_.Path }
if ($missingBuild) {
  Write-Error "缺少驱动构建必要工具，请安装 Visual Studio Build Tools + Windows WDK，或在 PATH 中暴露 msbuild/inf2cat。"
}
