import { useEffect, useMemo, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import type { AppConfig, DeviceList, GateMode, RuntimeStatus } from '@/lib/types';

const DEFAULT_CONFIG: AppConfig = {
  route: { input_device_id: '', bridge_output_device_id: '' },
  hotkey: { accelerator: 'Ctrl+Shift+V', mode: 'ptt' },
  launch_on_startup: false,
  minimize_to_tray: true,
};

function modeLabel(mode: GateMode): string {
  if (mode === 'ptt') return '按住说话';
  if (mode === 'toggle') return '切换开关';
  return '混合模式';
}

export default function App() {
  const [devices, setDevices] = useState<DeviceList>({ inputs: [], outputs: [] });
  const [config, setConfig] = useState<AppConfig>(DEFAULT_CONFIG);
  const [status, setStatus] = useState<RuntimeStatus | null>(null);
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState('');

  const canStart = useMemo(
    () => Boolean(config.route.input_device_id && config.route.bridge_output_device_id),
    [config.route.bridge_output_device_id, config.route.input_device_id],
  );

  const refresh = async () => {
    const [list, cfg, runtime] = await Promise.all([
      invoke<DeviceList>('list_audio_devices'),
      invoke<AppConfig>('get_app_config'),
      invoke<RuntimeStatus>('get_runtime_status'),
    ]);
    setDevices(list);
    setConfig(cfg);
    setStatus(runtime);
  };

  useEffect(() => {
    refresh().catch((e) => setMessage(String(e)));
    const timer = setInterval(() => {
      invoke<RuntimeStatus>('get_runtime_status')
        .then(setStatus)
        .catch(() => undefined);
    }, 1000);
    return () => clearInterval(timer);
  }, []);

  const save = async () => {
    setLoading(true);
    setMessage('');
    try {
      await invoke('save_audio_route', { config: config.route });
      await invoke('set_hotkey', { config: config.hotkey });
      await invoke('set_launch_on_startup', { enabled: config.launch_on_startup });
      await invoke('set_minimize_to_tray', { enabled: config.minimize_to_tray });
      setMessage('配置已保存');
      await refresh();
    } catch (e) {
      setMessage(`保存失败：${String(e)}`);
    } finally {
      setLoading(false);
    }
  };

  const start = async () => {
    setLoading(true);
    setMessage('');
    try {
      await invoke('start_engine');
      setMessage('音频桥接已启动');
      await refresh();
    } catch (e) {
      setMessage(`启动失败：${String(e)}`);
    } finally {
      setLoading(false);
    }
  };

  const stop = async () => {
    setLoading(true);
    setMessage('');
    try {
      await invoke('stop_engine');
      setMessage('音频桥接已停止');
      await refresh();
    } catch (e) {
      setMessage(`停止失败：${String(e)}`);
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
    <main className="min-h-screen p-4 bg-background text-foreground">
      <div className="mx-auto max-w-3xl space-y-4">
        <Card>
          <CardHeader>
            <CardTitle>Windows Mic Ctrl</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-sm">麦克风状态</span>
              <span className="text-sm font-medium">
                {status?.gate_state.is_open ? '开启' : '关闭'} / {status ? modeLabel(status.gate_state.mode) : '-'}
              </span>
            </div>
            <div className="flex gap-2">
              <Button onClick={toggleGate} disabled={!status || loading}>
                {status?.gate_state.is_open ? '立即闭麦' : '立即开麦'}
              </Button>
              <Button variant="outline" onClick={start} disabled={!canStart || loading}>
                启动桥接
              </Button>
              <Button variant="outline" onClick={stop} disabled={loading}>
                停止桥接
              </Button>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>音频设备</CardTitle>
          </CardHeader>
          <CardContent className="grid grid-cols-1 gap-3 md:grid-cols-2">
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
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div>
              <p className="mb-1 text-sm">虚拟麦克风桥接输出</p>
              <Select
                value={config.route.bridge_output_device_id}
                onValueChange={(value) => setConfig((prev) => ({ ...prev, route: { ...prev.route, bridge_output_device_id: value } }))}
              >
                <SelectTrigger>
                  <SelectValue placeholder="请选择输出设备" />
                </SelectTrigger>
                <SelectContent>
                  {devices.outputs.map((device) => (
                    <SelectItem key={device.id} value={device.id}>
                      {device.name}
                      {device.is_virtual_candidate ? '（推荐）' : ''}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>快捷键与行为</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <div>
              <p className="mb-1 text-sm">全局快捷键（字符串）</p>
              <input
                className="h-9 w-full rounded-lg border border-border bg-background px-3"
                value={config.hotkey.accelerator}
                onChange={(e) =>
                  setConfig((prev) => ({
                    ...prev,
                    hotkey: { ...prev.hotkey, accelerator: e.target.value },
                  }))
                }
              />
              <p className="mt-1 text-xs opacity-70">示例：Ctrl+Shift+V</p>
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
            <p>引擎状态：{status?.engine_state ?? '-'}</p>
            <p>缓冲水位：{status?.buffer_level_ms ?? 0} ms</p>
            <p>XRuns：{status?.xruns ?? 0}</p>
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
