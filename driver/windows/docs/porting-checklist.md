# SysVAD 派生移植清单（Windows Mic Ctrl）

## 目标

把 SysVAD 示例改造成可安装的 `Windows Mic Ctrl Virtual Mic` 录制端点驱动。

## 必做改造

1. 设备与厂商标识
   - 更新硬件 ID / 设备友好名
   - 统一 `Windows Mic Ctrl Virtual Mic` 命名

2. 端点能力
   - 录制端点（Capture）
   - 支持 48kHz/16bit/f32（根据用户态链路约束）

3. 用户态通信
   - 预留 IOCTL / 共享缓冲接口
   - 映射门控状态（开麦/闭麦）

4. INF 与安装
   - 对齐服务名 `windows_mic_ctrl_virtual_mic`
   - 验证安装、卸载、升级路径

5. 验证
   - 声音设置“录制”可见
   - QQ 可选中并有音频输入
