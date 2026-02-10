# Windows Mic Ctrl

Windows 麦克风按键控制工具（Rust + Tauri）。

## 功能

- 物理麦克风输入门控（PTT / Toggle / Hybrid）
- 全局快捷键按键录入（不再手输字符串）
- 启动即自动初始化语音链路
- 主界面仅保留物理麦克风输入选择
- 最小化到托盘，托盘支持切换开麦/闭麦与重初始化
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
2. 打开应用后会自动初始化语音链路。
3. 在主界面选择物理麦克风输入设备并保存。
4. 在 QQ 中将语音输入设备设置为虚拟声卡输出端（例如 `CABLE Output`）。
5. 点击“按键录入”，按下你的快捷键组合。

## 关于“程序自建虚拟麦克风”

当前仓库已新增 `driver/` 驱动工程骨架与接口协议文档，用于后续实现内核态虚拟麦端点。当前可运行版本仍采用兼容路径（依赖已安装的虚拟声卡设备）。

## 当前范围

- 目标平台：Windows 10/11 x64
- 首版不包含降噪/AEC/混音

## Release

- 当在 GitHub 发布版本（Release Published）时，仅构建 Windows 产物并上传到该 Release。
