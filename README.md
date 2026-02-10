# Windows Mic Ctrl

Windows 麦克风按键控制工具（Rust + Tauri）。

## 功能

- 物理麦克风到虚拟麦克风桥接（用于 QQ 语音等应用）
- 全局快捷键门控（PTT / Toggle / Hybrid）
- 最小化到托盘，托盘快捷开关麦克风
- 开机自启动开关
- 运行状态诊断（缓冲、XRuns、错误）

## 技术栈

- 前端：React + TypeScript + Tailwind + shadcn 风格组件
- 后端：Rust + Tauri v2
- 音频：CPAL（WASAPI）

## 快速开始

```bash
npm install
npm run dev
```

## 首次使用

1. 安装虚拟声卡（例如 VB-CABLE）。
2. 在本应用中选择：
   - 输入设备：物理麦克风
   - 桥接输出：虚拟声卡输入端（例如 `CABLE Input`）
3. 在 QQ 中将语音输入设备设置为虚拟声卡输出端（例如 `CABLE Output`）。
4. 设置按键模式与全局快捷键。

## 当前范围

- 目标平台：Windows 10/11 x64
- 首版不包含降噪/AEC/混音


## CI / Release

- 仅 `Release Build`：当你在 GitHub 发布一个版本（Release Published）时，自动构建 Windows / macOS / Linux 可执行包并上传到该 Release。

示例发布流程：

1. 推送代码到 `main`。
2. 在 GitHub 页面创建 Release（例如 `v0.1.0`）。
3. Actions 自动开始三平台构建，并把产物回传到该 Release。
