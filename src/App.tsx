import { useCallback, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import type { AppConfig, DeviceList, GateMode, RuntimeStatus, VirtualMicStatus } from '@/lib/types';

const DEFAULT_CONFIG: AppConfig = {
  route: { input_device_id: '', bridge_output_device_id: '' },
  hotkey: { accelerator: 'Ctrl+Shift+V', mode: 'ptt' },
  launch_on_startup: false,
  minimize_to_tray: true,
};

const MODIFIER_KEYS = new Set(['Control', 'Shift', 'Alt', 'Meta']);

function modeLabel(mode: GateMode): string {
  if (mode === 'ptt') return '按住说话';
  if (mode === 'toggle') return '切换开关';
  return '混合模式';
}

function engineLabel(state?: RuntimeStatus['engine_state']): string {
  if (state === 'running') return '已就绪';
  if (state === 'error') return '错误';
  return '未就绪';
}

function normalizeMainKey(event: KeyboardEvent): string | null {
  const { code, key } = event;

  if (code.startsWith('Key') && code.length === 4) {
    return code.slice(3).toUpperCase();
  }
  if (code.startsWith('Digit') && code.length === 6) {
    return code.slice(5);
  }

  if (/^F\d{1,2}$/i.test(key)) {
    return key.toUpperCase();
  }

  const map: Record<string, string> = {
    ' ': 'Space',
    Spacebar: 'Space',
    Escape: 'Esc',
    Enter: 'Enter',
    Tab: 'Tab',
    Backspace: 'Backspace',
    Delete: 'Delete',
    Insert: 'Insert',
    Home: 'Home',
    End: 'End',
    PageUp: 'PageUp',
    PageDown: 'PageDown',
    ArrowUp: 'Up',
    ArrowDown: 'Down',
    ArrowLeft: 'Left',
    ArrowRight: 'Right',
  };

  if (map[key]) return map[key];

  if (key.length === 1) {
    return key.toUpperCase();
  }

  return null;
}

function buildAccelerator(event: KeyboardEvent): string | null {
  const parts: string[] = [];
  if (event.ctrlKey) parts.push('Ctrl');
  if (event.altKey) parts.push('Alt');
  if (event.shiftKey) parts.push('Shift');
  if (event.metaKey) parts.push('Super');

  if (MODIFIER_KEYS.has(event.key)) {
    return null;
  }

  const mainKey = normalizeMainKey(event);
  if (!mainKey) {
    return null;
  }

  if (['Ctrl', 'Alt', 'Shift', 'Super'].includes(mainKey)) {
    return null;
  }

  return [...parts, mainKey].join('+');
}

export default function App() {
  const [devices, setDevices] = useState<DeviceList>({ inputs: [], outputs: [] });
  const [config, setConfig] = useState<AppConfig>(DEFAULT_CONFIG);
  const [status, setStatus] = useState<RuntimeStatus | null>(null);
  const [virtualMic, setVirtualMic] = useState<VirtualMicStatus | null>(null);
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState('');
  const [recordingHotkey, setRecordingHotkey] = useState(false);

  const refresh = useCallback(async () => {
    const [list, cfg, runtime, vmStatus] = await Promise.all([
      invoke<DeviceList>('list_audio_devices'),
      invoke<AppConfig>('get_app_config'),
      invoke<RuntimeStatus>('get_runtime_status'),
      invoke<VirtualMicStatus>('get_virtual_mic_status'),
    ]);
    setDevices(list);
    setConfig(cfg);
    setStatus(runtime);
    setVirtualMic(vmStatus);
  }, []);

  useEffect(() => {
    refresh().catch((e) => setMessage(String(e)));
    const timer = setInterval(() => {
      invoke<RuntimeStatus>('get_runtime_status')
        .then(setStatus)
        .catch(() => undefined);
    }, 1000);
    return () => clearInterval(timer);
  }, [refresh]);

  useEffect(() => {
    if (!recordingHotkey) return;

    const onKeyDown = (event: KeyboardEvent) => {
      event.preventDefault();
      event.stopPropagation();

      if (event.key === 'Escape') {
        setRecordingHotkey(false);
        setMessage('已取消快捷键录入');
        return;
      }

      const accelerator = buildAccelerator(event);
      if (!accelerator) {
        setMessage('请按下至少一个修饰键 + 主键（例如 Ctrl+Shift+V）');
        return;
      }

      setConfig((prev) => ({
        ...prev,
        hotkey: {
          ...prev.hotkey,
          accelerator,
        },
      }));
      setRecordingHotkey(false);
      setMessage(`已录入快捷键：${accelerator}`);
    };

    window.addEventListener('keydown', onKeyDown, true);
    return () => window.removeEventListener('keydown', onKeyDown, true);
  }, [recordingHotkey]);

  const reinitializeEngine = async () => {
    setLoading(true);
    setMessage('');
    try {
      await invoke('stop_engine');
      await invoke('start_engine');
      await refresh();
      setMessage('语音链路已重新初始化');
    } catch (e) {
      setMessage(`初始化失败：${String(e)}`);
      await refresh().catch(() => undefined);
    } finally {
      setLoading(false);
    }
  };

  const save = async () => {
    setLoading(true);
    setMessage('');
    try {
      await invoke('save_audio_route', { config: config.route });
      await invoke('set_hotkey', { config: config.hotkey });
      await invoke('set_launch_on_startup', { enabled: config.launch_on_startup });
      await invoke('set_minimize_to_tray', { enabled: config.minimize_to_tray });
      await invoke('stop_engine');
      await invoke('start_engine');
      setMessage('配置已保存并自动重新初始化');
      await refresh();
    } catch (e) {
      setMessage(`保存失败：${String(e)}`);
      await refresh().catch(() => undefined);
    } finally {
      setLoading(false);
    }
  };

  const toggleGate = async () => {
    if (!status) return;
    setLoading(true);
    try {
      await invoke('set_mic_gate', { open: !status.gate_state.is_open, source: 'ui' });
      await refresh();
    } catch (e) {
      setMessage(`切换失败：${String(e)}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <main className="min-h-screen bg-background p-4 text-foreground">
      <div className="mx-auto max-w-3xl space-y-4">
        <Card>
          <CardHeader>
            <CardTitle>Windows Mic Ctrl</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-sm">当前状态</span>
              <span className="text-sm font-medium">{status?.gate_state.is_open ? '开麦' : '闭麦'}</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm">按键模式</span>
              <span className="text-sm font-medium">{status ? modeLabel(status.gate_state.mode) : '-'}</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm">语音链路</span>
              <span className="text-sm font-medium">{engineLabel(status?.engine_state)}</span>
            </div>
            <div className="flex flex-wrap gap-2">
              <Button onClick={toggleGate} disabled={!status || loading}>
                {status?.gate_state.is_open ? '切换到闭麦' : '切换到开麦'}
              </Button>
              <Button variant="outline" onClick={reinitializeEngine} disabled={loading}>
                重新初始化语音链路
              </Button>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>设备设置</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <div>
              <p className="mb-1 text-sm">物理麦克风输入</p>
              <Select
                value={config.route.input_device_id}
                onValueChange={(value) => setConfig((prev) => ({ ...prev, route: { ...prev.route, input_device_id: value } }))}
              >
                <SelectTrigger>
                  <SelectValue placeholder="请选择输入设备" />
                </SelectTrigger>
                <SelectContent>
                  {devices.inputs.map((device) => (
                    <SelectItem key={device.id} value={device.id}>
                      {device.name}
                      {device.is_default ? '（系统默认）' : ''}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <p className="mt-1 text-xs opacity-70">虚拟麦克风端点由程序自动初始化与管理。</p>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>快捷键与行为</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <div>
              <p className="mb-1 text-sm">全局快捷键</p>
              <div className="flex gap-2">
                <input
                  className="h-9 flex-1 rounded-lg border border-border bg-background px-3"
                  value={config.hotkey.accelerator}
                  readOnly
                />
                <Button
                  variant="outline"
                  onClick={() => {
                    setRecordingHotkey((prev) => !prev);
                    setMessage('');
                  }}
                  disabled={loading}
                >
                  {recordingHotkey ? '取消录入' : '按键录入'}
                </Button>
              </div>
              <p className="mt-1 text-xs opacity-70">
                {recordingHotkey ? '请按下快捷键组合，按 Esc 取消。' : '点击“按键录入”，直接按下组合键完成配置。'}
              </p>
            </div>

            <div>
              <p className="mb-1 text-sm">按键模式</p>
              <Select
                value={config.hotkey.mode}
                onValueChange={(value) =>
                  setConfig((prev) => ({
                    ...prev,
                    hotkey: { ...prev.hotkey, mode: value as GateMode },
                  }))
                }
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="ptt">按住说话</SelectItem>
                  <SelectItem value="toggle">切换开关</SelectItem>
                  <SelectItem value="hybrid">混合模式</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div className="flex items-center justify-between">
              <span className="text-sm">开机启动</span>
              <Switch
                checked={config.launch_on_startup}
                onCheckedChange={(checked) => setConfig((prev) => ({ ...prev, launch_on_startup: checked }))}
              />
            </div>

            <div className="flex items-center justify-between">
              <span className="text-sm">关闭窗口最小化到托盘</span>
              <Switch
                checked={config.minimize_to_tray}
                onCheckedChange={(checked) => setConfig((prev) => ({ ...prev, minimize_to_tray: checked }))}
              />
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>运行诊断</CardTitle>
          </CardHeader>
          <CardContent className="space-y-1 text-sm">
            <p>链路状态：{engineLabel(status?.engine_state)}</p>
            <p>缓冲水位：{status?.buffer_level_ms ?? 0} ms</p>
            <p>XRuns：{status?.xruns ?? 0}</p>
            <p>虚拟麦后端：{virtualMic ? `${virtualMic.backend}（${virtualMic.ready ? '就绪' : '未就绪'}）` : '-'}</p>
            <p>后端详情：{virtualMic?.detail ?? '-'}</p>
            <p>最近错误：{status?.last_error ?? '无'}</p>
          </CardContent>
        </Card>

        <div className="flex items-center gap-2">
          <Button onClick={save} disabled={loading}>
            保存配置
          </Button>
          <Button variant="outline" onClick={() => refresh().catch((e) => setMessage(String(e)))} disabled={loading}>
            刷新
          </Button>
          <span className="text-sm opacity-80">{message}</span>
        </div>
      </div>
    </main>
  );
}
