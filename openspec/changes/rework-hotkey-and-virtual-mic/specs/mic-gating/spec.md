# mic-gating Delta Spec (rework-hotkey-and-virtual-mic)

## ADDED Requirements

### Requirement: 快捷键录入控件
系统 MUST 提供按键录入控件来采集快捷键组合，而非仅允许字符串手动输入。

#### Scenario: 用户录入组合键
- Given 用户打开快捷键设置项
- When 用户按下组合键
- Then 系统 SHALL 捕获并展示标准化快捷键
- And SHALL 尝试注册该快捷键并反馈结果

### Requirement: 启动自动初始化
系统 MUST 在应用启动时自动初始化语音链路，并明确展示初始化结果。

#### Scenario: 自动初始化成功
- Given 应用启动
- When 环境满足初始化条件
- Then 语音链路 SHALL 自动进入可用状态

#### Scenario: 自动初始化失败
- Given 应用启动
- When 依赖缺失或设备不可用
- Then 系统 SHALL 显式提示失败原因并记录日志

### Requirement: 简化设备配置
系统 MUST 在主界面仅暴露物理麦克风输入设备选择。

#### Scenario: 用户配置设备
- Given 用户打开设备配置
- When 用户查看设备项
- Then 系统 SHALL 仅展示物理输入设备列表

## MODIFIED Requirements

### Requirement: 虚拟麦后端分阶段交付
系统 MUST 提供“虚拟麦后端初始化状态”并在后端未就绪时显式提示；在后端未就绪阶段，系统 SHALL 允许兼容路径继续运行（外部虚拟声卡）。

#### Scenario: 后端未就绪
- Given 应用启动
- When 自建虚拟麦驱动未就绪
- Then 系统 SHALL 在诊断区展示未就绪原因
- And SHALL 允许使用兼容路径继续进行按键门控

#### Scenario: 后端就绪（后续版本）
- Given 自建虚拟麦驱动已安装且初始化完成
- When 用户打开目标语音应用输入设备列表
- Then SHALL 出现本应用提供的虚拟麦端点

### Requirement: 门控状态文案
系统 MUST 使用可理解文案表达门控状态与模式。

#### Scenario: 查看门控状态
- Given 用户在主界面或托盘查看状态
- When 门控状态变化
- Then 文案 SHALL 表达为“当前状态：开麦/闭麦”与“模式：按住说话/切换开关”

## REMOVED Requirements

### Requirement: 桥接输出设备手动选择
系统 MUST NOT 在主界面要求用户手动配置桥接输出设备。

#### Scenario: 主界面查看
- Given 用户打开主界面
- When 查看设备配置
- Then 不应出现桥接输出设备选择项

### Requirement: 发布构建包含驱动离线包
系统 MUST 在发布构建阶段打包并附带虚拟麦驱动产物（`.sys/.inf/.cat`），以支持离线安装。

#### Scenario: 发布构建执行
- Given 驱动产物目录已准备完整文件
- When 执行发布构建流程
- Then 系统 SHALL 将驱动产物打包进安装包资源
- And SHALL 输出独立驱动离线包用于排障安装
