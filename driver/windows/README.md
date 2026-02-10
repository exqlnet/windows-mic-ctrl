# Windows 虚拟麦驱动工程（方案 A）

本目录用于落地“应用自身创建虚拟麦克风端点（录制设备）”的 Windows 驱动实现。

## 目标结果

安装驱动后，系统「声音设置 -> 录制」出现：

- `Windows Mic Ctrl Virtual Mic`

并可被 QQ 等语音应用选择为输入设备。

## 当前阶段

### Phase 1（已完成）

- [x] 目录结构、INF 模板、安装脚本基础版
- [x] 应用侧虚拟麦端点检测（真实枚举录制设备）

### Phase 2（已完成）

- [x] SysVAD 拉取脚本（`scripts/prepare-sysvad.ps1`）
- [x] 测试安装/卸载脚本（`install-driver-test.ps1` / `uninstall-driver.ps1`）
- [x] 工具链检查脚本（`check-toolchain.ps1`）
- [x] 派生参数清单脚本（`apply-porting-overrides.ps1`）
- [x] 一键开发引导脚本（`dev-bootstrap.ps1`）

### Phase 3（进行中）

- [ ] 内核态虚拟音频端点实现（SysVAD 派生代码）
- [ ] 用户态 IOCTL / 共享缓冲写入链路
- [ ] 自动化打包测试签名驱动

## 目录说明

- `inf/`：驱动 INF 模板
- `scripts/`：构建/安装/卸载/源码准备脚本
- `docs/`：移植清单与开发文档
- `src/`：驱动源代码（后续接入 SysVAD 派生实现）

## 一次性执行流程

1. `scripts/dev-bootstrap.ps1`
2. 按 `docs/porting-checklist.md` 修改 SysVAD 代码
3. `scripts/build-driver.ps1`
4. `scripts/install-driver-test.ps1`

详见：`docs/build-and-install.md`

## 签名与安装约束

- 开发阶段：可用测试签名（Test Mode）
- 发布阶段：需正式签名（EV/WHQL 路线）
- 安装驱动需管理员权限

## 与应用发布集成

应用发布构建命令 `npm run build:release` 会调用 `scripts/stage-driver-assets.mjs`，将 `driver/windows/artifacts/driver` 中的驱动产物拷贝到 `src-tauri/drivers/windows`，并随安装包一起发布。
