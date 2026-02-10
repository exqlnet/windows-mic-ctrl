# Windows 虚拟麦驱动：构建与安装流程

## 0. 一键准备（推荐）

在管理员 PowerShell 中执行：

- `driver/windows/scripts/dev-bootstrap.ps1`

该脚本会依次执行：

1. `check-toolchain.ps1`（检查 VS Build Tools / WDK / pnputil 等）
2. `prepare-sysvad.ps1`（拉取并准备 SysVAD 源码）
3. `apply-porting-overrides.ps1`（生成派生参数清单）

## 1. 完成 SysVAD 派生改造

按 `docs/porting-checklist.md` 修改源码，重点包括：

- 设备友好名：`Windows Mic Ctrl Virtual Mic`
- 服务名：`windows_mic_ctrl_virtual_mic`
- 硬件 ID：`ROOT\\WINDOWS_MIC_CTRL_VIRTUAL_MIC`

## 2. 构建驱动

- `driver/windows/scripts/build-driver.ps1`

产物默认输出到：`driver/windows/artifacts/driver`

## 3. 测试安装

- `driver/windows/scripts/install-driver-test.ps1`

若提示需要 Test Mode，可先执行：

- `driver/windows/scripts/install-driver-test.ps1 -EnableTestSigning`
- 重启系统后再次执行安装命令。

## 4. 验证

- Windows 声音设置 -> 录制：应出现 `Windows Mic Ctrl Virtual Mic`
- QQ 语音输入设备可选择该端点。

## 5. 卸载

- `driver/windows/scripts/uninstall-driver.ps1`

若需彻底移除驱动包，按脚本提示执行 `pnputil /delete-driver`。

## 6. 打包到应用安装包（离线）

完成驱动构建后，把产物放到 `driver/windows/artifacts/driver`，再在项目根目录执行：

- `npm run build:release`

该命令会自动校验并拷贝 `.sys/.inf/.cat` 到 `src-tauri/drivers/windows`，并在 Tauri 打包时写入安装包资源。发布工作流也会额外上传 `windows-driver-package.zip` 供离线排障安装。

## 7. 应用启动自动安装行为

从 `v0.1.9` 起，应用在 Windows 启动时会自动执行：

1. 检查录制端点是否已有 `Windows Mic Ctrl Virtual Mic`
2. 若缺失，尝试从安装包内 `drivers/windows/*.inf` 触发驱动安装（UAC）
3. 安装后轮询验证端点是否出现

若安装失败，应用会在诊断信息中显示失败原因（例如：拒绝 UAC、签名不满足策略、需要重启）。

## 8. CI 环境注意项

- `prepare-sysvad.ps1` 现会同步拉取上游 `wil/` 目录，避免 APO 项目缺少 `wil/com.h`。
- `build-driver.ps1` 默认关闭 INF 验证并在失败时尝试回收已生成产物，便于 CI 阶段先完成打包联调。
