import { useCallback, useEffect, useRef, useState } from 'react';
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

function configSignature(config: AppConfig): string {
  return JSON.stringify(config);
}

function normalizeMainKey(event: KeyboardEvent): string | null {
  const { code, key } = event;

  if (code.startsWith('Key') && code.length === 4) return code.slice(3).toUpperCase();
  if (code.startsWith('Digit') && code.length === 6) return code.slice(5);
  if (/^F\d{1,2}$/i.test(key)) return key.toUpperCase();

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
  if (key.length === 1) return key.toUpperCase();
  return null;
}

function keyboardAccelerator(event: KeyboardEvent): string | null {
  const parts: string[] = [];
  if (event.ctrlKey) parts.push('Ctrl');
  if (event.altKey) parts.push('Alt');
  if (event.shiftKey) parts.push('Shift');
  if (event.metaKey) parts.push('Super');

  if (MODIFIER_KEYS.has(event.key)) return null;

  const mainKey = normalizeMainKey(event);
  if (!mainKey) return null;
  if (['Ctrl', 'Alt', 'Shift', 'Super'].includes(mainKey)) return null;

  return [...parts, mainKey].join('+');
}

function mouseAccelerator(event: MouseEvent): string {
  const parts: string[] = [];
  if (event.ctrlKey) parts.push('Ctrl');
  if (event.altKey) parts.push('Alt');
  if (event.shiftKey) parts.push('Shift');
  if (event.metaKey) parts.push('Super');

  const buttonMap: Record<number, string> = {
    0: 'MouseLeft',
    1: 'MouseMiddle',
    2: 'MouseRight',
    3: 'MouseBack',
    4: 'MouseForward',
  };

  const main = buttonMap[event.button] ?? `Mouse${event.button}`;
  return [...parts, main].join('+');
}

export default function App() {
  const [devices, setDevices] = useState<DeviceList>({ inputs: [], outputs: [] });
  const [config, setConfig] = useState<AppConfig>(DEFAULT_CONFIG);
  const [status, setStatus] = useState<RuntimeStatus | null>(null);
  const [virtualMic, setVirtualMic] = useState<VirtualMicStatus | null>(null);
  const [message, setMessage] = useState('');
  const [recordingHotkey, setRecordingHotkey] = useState(false);
  const [bootstrapped, setBootstrapped] = useState(false);
  const [autoSaving, setAutoSaving] = useState(false);

  const lastSavedSignatureRef = useRef('');
  const lastSavedConfigRef = useRef<AppConfig>(DEFAULT_CONFIG);
  const recordingStartAtRef = useRef(0);

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
    lastSavedConfigRef.current = cfg;
    lastSavedSignatureRef.current = configSignature(cfg);
    setBootstrapped(true);
  }, []);

  useEffect(() => {
    refresh().catch((error) => setMessage(String(error)));
    const timer = setInterval(() => {
      invoke<RuntimeStatus>('get_runtime_status').then(setStatus).catch(() => undefined);
    }, 1000);
    return () => clearInterval(timer);
  }, [refresh]);

  const beginHotkeyRecording = () => {
    recordingStartAtRef.current = Date.now();
    setRecordingHotkey(true);
    setMessage('Press key...');
  };

  useEffect(() => {
    if (!recordingHotkey) return;

    const finishRecording = (accelerator: string) => {
      setConfig((previous) => ({
        ...previous,
        hotkey: {
          ...previous.hotkey,
          accelerator,
        },
      }));
      setRecordingHotkey(false);
      setMessage(`已录入快捷键：${accelerator}`);
    };

    const onKeyDown = (event: KeyboardEvent) => {
      event.preventDefault();
      event.stopPropagation();

      if (event.key === 'Escape') {
        setRecordingHotkey(false);
        setMessage('已取消快捷键录入');
        return;
      }

      const accelerator = keyboardAccelerator(event);
      if (!accelerator) {
        setMessage('请按下有效快捷键组合');
        return;
      }

      finishRecording(accelerator);
    };

    const onMouseDown = (event: MouseEvent) => {
      event.preventDefault();
      event.stopPropagation();

      if (Date.now() - recordingStartAtRef.current < 180) {
        return;
      }

      const accelerator = mouseAccelerator(event);
      finishRecording(accelerator);
    };

    const onContextMenu = (event: MouseEvent) => {
      event.preventDefault();
    };

    window.addEventListener('keydown', onKeyDown, true);
    window.addEventListener('mousedown', onMouseDown, true);
    window.addEventListener('contextmenu', onContextMenu, true);

    return () => {
      window.removeEventListener('keydown', onKeyDown, true);
      window.removeEventListener('mousedown', onMouseDown, true);
      window.removeEventListener('contextmenu', onContextMenu, true);
    };
  }, [recordingHotkey]);

  useEffect(() => {
    if (!bootstrapped) return;

    const nextSignature = configSignature(config);
    if (nextSignature === lastSavedSignatureRef.current) return;

    const timer = setTimeout(async () => {
      setAutoSaving(true);
      const previous = lastSavedConfigRef.current;

      try {
        await invoke('save_audio_route', { config: config.route });
        await invoke('set_hotkey', { config: config.hotkey });
        await invoke('set_launch_on_startup', { enabled: config.launch_on_startup });
        await invoke('set_minimize_to_tray', { enabled: config.minimize_to_tray });

        const routeChanged =
          previous.route.input_device_id !== config.route.input_device_id ||
          previous.route.bridge_output_device_id !== config.route.bridge_output_device_id;

        if (routeChanged) {
          await invoke('stop_engine');
          await invoke('start_engine');
        }

        const [runtime, vmStatus] = await Promise.all([
          invoke<RuntimeStatus>('get_runtime_status'),
          invoke<VirtualMicStatus>('get_virtual_mic_status'),
        ]);

        setStatus(runtime);
        setVirtualMic(vmStatus);

        lastSavedConfigRef.current = config;
        lastSavedSignatureRef.current = nextSignature;
        setMessage('配置已自动保存');
      } catch (error) {
        setMessage(`自动保存失败：${String(error)}`);
      } finally {
        setAutoSaving(false);
      }
    }, 450);

    return () => clearTimeout(timer);
  }, [bootstrapped, config]);

  const reinitializeEngine = async () => {
    setAutoSaving(true);
    try {
      await invoke('stop_engine');
      await invoke('start_engine');
      const [runtime, vmStatus] = await Promise.all([
        invoke<RuntimeStatus>('get_runtime_status'),
        invoke<VirtualMicStatus>('get_virtual_mic_status'),
      ]);
      setStatus(runtime);
      setVirtualMic(vmStatus);
      setMessage('语音链路已重新初始化');
    } catch (error) {
      setMessage(`重新初始化失败：${String(error)}`);
    } finally {
      setAutoSaving(false);
    }
  };

  return (
    <main className="min-h-screen bg-background p-4 text-foreground">
      <div className="mx-auto max-w-3xl space-y-4">
        <Card>
          <CardHeader>
            <CardTitle>设备设置</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <div>
              <p className="mb-1 text-sm">物理麦克风输入</p>
              <Select
                value={config.route.input_device_id}
                onValueChange={(value) =>
                  setConfig((previous) => ({
                    ...previous,
                    route: { ...previous.route, input_device_id: value },
                  }))
                }
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
            </div>

            <div className="flex items-center justify-between">
              <span className="text-sm">语音链路状态：{engineLabel(status?.engine_state)}</span>
              <Button variant="outline" onClick={reinitializeEngine} disabled={autoSaving}>
                重新初始化语音链路
              </Button>
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
              <input
                className="h-9 w-full rounded-lg border border-border bg-background px-3"
                value={recordingHotkey ? 'Press key...' : config.hotkey.accelerator}
                readOnly
                onFocus={beginHotkeyRecording}
                onClick={beginHotkeyRecording}
              />
              <p className="mt-1 text-xs opacity-70">点击输入框后按键录入，支持键盘组合与鼠标按键。</p>
            </div>

            <div>
              <p className="mb-1 text-sm">按键模式</p>
              <Select
                value={config.hotkey.mode}
                onValueChange={(value) =>
                  setConfig((previous) => ({
                    ...previous,
                    hotkey: { ...previous.hotkey, mode: value as GateMode },
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
                onCheckedChange={(checked) =>
                  setConfig((previous) => ({
                    ...previous,
                    launch_on_startup: checked,
                  }))
                }
              />
            </div>

            <div className="flex items-center justify-between">
              <span className="text-sm">关闭窗口最小化到托盘</span>
              <Switch
                checked={config.minimize_to_tray}
                onCheckedChange={(checked) =>
                  setConfig((previous) => ({
                    ...previous,
                    minimize_to_tray: checked,
                  }))
                }
              />
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>运行诊断</CardTitle>
          </CardHeader>
          <CardContent className="space-y-1 text-sm">
            <p>按键模式：{status ? modeLabel(status.gate_state.mode) : '-'}</p>
            <p>链路状态：{engineLabel(status?.engine_state)}</p>
            <p>缓冲水位：{status?.buffer_level_ms ?? 0} ms</p>
            <p>XRuns：{status?.xruns ?? 0}</p>
            <p>虚拟麦后端：{virtualMic ? `${virtualMic.backend}（${virtualMic.ready ? '就绪' : '未就绪'}）` : '-'}</p>
            <p>后端详情：{virtualMic?.detail ?? '-'}</p>
            <p>最近错误：{status?.last_error ?? '无'}</p>
          </CardContent>
        </Card>

        <p className="text-sm opacity-80">{autoSaving ? '正在自动保存配置...' : message}</p>
      </div>
    </main>
  );
}
