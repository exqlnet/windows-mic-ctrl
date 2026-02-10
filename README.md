# Windows Mic Ctrl

Windows 麦克风按键控制工具（Rust + Tauri）。

## 功能

- 物理麦克风输入门控（PTT / Toggle / Hybrid）
- 全局快捷键录入（点击输入框后直接按键，支持键盘组合与鼠标按键）
- 启动后自动初始化语音链路（配置变更自动保存并自动应用）
- 每次启动自动检查虚拟麦驱动，未安装时自动触发安装（UAC 提权）
- 主界面仅保留物理麦克风输入选择
- 关闭窗口可最小化到托盘（托盘仅保留状态/显示主界面/退出）
- 运行诊断（缓冲、XRuns、最近错误、虚拟麦驱动状态）

## 技术栈

- 前端：React + TypeScript + Tailwind + shadcn 风格组件
- 后端：Rust + Tauri v2
- 音频：CPAL（WASAPI）

## 快速开始

```bash
npm install
npm run dev
```

## 发布构建（含驱动打包）

在执行发布构建前，请先准备驱动产物目录：`driver/windows/artifacts/driver`，至少包含：

- `.sys`
- `.inf`
- `.cat`

然后执行：

```bash
npm run build:release
```

该流程会自动执行：

1. `npm run stage:driver`：校验并拷贝驱动文件到 `src-tauri/drivers/windows`
2. `tauri build`：将驱动文件随安装包资源一起打包

## 开箱即用条件（Windows）

若希望用户“安装后直接可用”，需要同时满足：

1. 安装包已内置可安装驱动（`.sys/.inf/.cat`）
2. 驱动签名满足目标环境（测试签名或正式签名）
3. 首次启动允许 UAC 提权完成 `pnputil` 安装

当前应用已实现“每次启动自动检查并在缺失时自动安装”，但驱动是否能安装成功仍取决于驱动产物与签名。

## 关于“程序自建虚拟麦克风”

仓库已包含 `driver/windows/` 驱动工程化骨架与脚本（SysVAD 派生路线）。

当前状态：

- 应用已能真实检测系统中是否存在 `Windows Mic Ctrl Virtual Mic` 录制端点。
- 若未检测到，会自动尝试安装安装包内驱动；失败时在诊断区给出原因。
- GitHub Release 流程会先尝试构建驱动产物，再按可用性决定是否打包驱动。

可参考：`driver/windows/docs/build-and-install.md`

## 当前范围

- 目标平台：Windows 10/11 x64
- 首版不包含降噪/AEC/混音

## Release

- 当 GitHub 发布版本（Release Published）时：
  - Workflow 会先尝试构建 `driver/windows/artifacts/driver`。
  - 若检测到完整驱动产物（`.sys/.inf/.cat`），执行 `npm run build:release` 并上传驱动离线包。
  - 若驱动产物未就绪，则自动降级为 `npm run build`，仅发布应用安装包。
