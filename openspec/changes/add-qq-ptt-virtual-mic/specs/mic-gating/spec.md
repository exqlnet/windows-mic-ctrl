# mic-gating Delta Spec

## ADDED Requirements

### Requirement: 音频桥接门控
系统 MUST 提供物理麦克风到虚拟输出设备的实时桥接，并可按门控状态输出音频或静音。

#### Scenario: 开麦透传
- Given 引擎已启动且输入输出设备有效
- When 门控状态为开启
- Then 输出设备 SHALL 接收输入音频帧

#### Scenario: 闭麦静音
- Given 引擎已启动
- When 门控状态为关闭
- Then 输出设备 SHALL 接收静音帧

### Requirement: 全局按键控制
系统 MUST 支持全局快捷键控制门控状态。

#### Scenario: PTT 模式
- Given 模式为 PTT
- When 用户按下快捷键
- Then 门控 SHALL 置为开启
- When 用户释放快捷键
- Then 门控 SHALL 置为关闭

### Requirement: 托盘可操作
系统 MUST 在托盘提供显示主窗口、切换麦克风状态和退出应用操作。

#### Scenario: 托盘切换门控
- Given 应用在托盘运行
- When 用户点击“开麦/闭麦”
- Then 门控状态 SHALL 立即更新

## MODIFIED Requirements

### Requirement: 配置持久化
系统 MUST 在重启后保留设备路由、快捷键和门控模式配置。

#### Scenario: 重启恢复
- Given 用户已保存配置
- When 应用重启
- Then 应读取并应用历史配置

## REMOVED Requirements

- 无
