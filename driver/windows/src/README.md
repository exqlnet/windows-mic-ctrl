# 驱动源码目录

## 目录布局

- `upstream/`：由 `prepare-sysvad.ps1` 拉取的 SysVAD 示例源码
- `windows-mic-ctrl-overrides/`：自动生成的派生参数清单（由 `apply-porting-overrides.ps1` 生成）

## 目标

在 SysVAD 派生实现中提供可被系统识别的录制端点：

- 名称：`Windows Mic Ctrl Virtual Mic`
- 服务：`windows_mic_ctrl_virtual_mic`
- 硬件 ID：`ROOT\\WINDOWS_MIC_CTRL_VIRTUAL_MIC`
