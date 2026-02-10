# Windows 驱动打包目录

本目录用于 Tauri 构建阶段打包驱动产物（`.sys` / `.inf` / `.cat`）。

## 生成方式

执行以下命令后会自动写入本目录：

- `npm run stage:driver`
- 或 `npm run build:release`（内含 `stage:driver`）

## 说明

- 本目录文件由脚本生成，不建议手动编辑。
- `driver-package-manifest.json` 用于记录本次打包的驱动文件来源与大小。
