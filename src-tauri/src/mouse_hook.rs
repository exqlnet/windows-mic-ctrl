use std::sync::Arc;

use crate::{error::AppError, gate::GateController, types::GateMode};

pub fn is_mouse_accelerator(accelerator: &str) -> bool {
    accelerator
        .split('+')
        .map(str::trim)
        .any(|token| token.to_ascii_lowercase().starts_with("mouse"))
}

#[cfg(target_os = "windows")]
mod imp {
    use std::{
        sync::{mpsc, OnceLock},
        thread,
        time::Duration,
    };

    use parking_lot::Mutex;
    use tauri::Emitter;
    use windows_sys::Win32::{
        Foundation::{LPARAM, LRESULT, WPARAM},
        System::{LibraryLoader::GetModuleHandleW, Threading::GetCurrentThreadId},
        UI::{
            Input::KeyboardAndMouse::{
                GetAsyncKeyState, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
            },
            WindowsAndMessaging::{
                CallNextHookEx, DispatchMessageW, GetMessageW, PostThreadMessageW,
                SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, HC_ACTION,
                MSLLHOOKSTRUCT, WH_MOUSE_LL, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN,
                WM_MBUTTONUP, WM_QUIT, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_XBUTTONDOWN, WM_XBUTTONUP,
                XBUTTON1, XBUTTON2,
            },
        },
    };

    use crate::{
        error::AppError, gate::GateController, mouse_hook::is_mouse_accelerator, types::GateMode,
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum MouseButton {
        Left,
        Right,
        Middle,
        Back,
        Forward,
    }

    #[derive(Debug, Clone)]
    struct MouseBinding {
        ctrl: bool,
        alt: bool,
        shift: bool,
        meta: bool,
        button: MouseButton,
    }

    #[derive(Clone)]
    struct HookContext {
        binding: MouseBinding,
        gate: std::sync::Arc<GateController>,
        app: tauri::AppHandle,
        mode: GateMode,
    }

    struct MouseHookWorker {
        thread_id: u32,
        join_handle: Option<thread::JoinHandle<()>>,
    }

    static HOOK_CONTEXT: OnceLock<Mutex<Option<HookContext>>> = OnceLock::new();

    fn hook_context() -> &'static Mutex<Option<HookContext>> {
        HOOK_CONTEXT.get_or_init(|| Mutex::new(None))
    }

    fn parse_mouse_binding(accelerator: &str) -> Result<MouseBinding, AppError> {
        if !is_mouse_accelerator(accelerator) {
            return Err(AppError::Hotkey("该快捷键不是鼠标按键格式".to_string()));
        }

        let mut binding = MouseBinding {
            ctrl: false,
            alt: false,
            shift: false,
            meta: false,
            button: MouseButton::Left,
        };

        let mut has_button = false;

        for token in accelerator
            .split('+')
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            let lower = token.to_ascii_lowercase();
            match lower.as_str() {
                "ctrl" | "control" => binding.ctrl = true,
                "alt" => binding.alt = true,
                "shift" => binding.shift = true,
                "super" | "win" | "meta" => binding.meta = true,
                "mouseleft" => {
                    binding.button = MouseButton::Left;
                    has_button = true;
                }
                "mouseright" => {
                    binding.button = MouseButton::Right;
                    has_button = true;
                }
                "mousemiddle" => {
                    binding.button = MouseButton::Middle;
                    has_button = true;
                }
                "mouseback" => {
                    binding.button = MouseButton::Back;
                    has_button = true;
                }
                "mouseforward" => {
                    binding.button = MouseButton::Forward;
                    has_button = true;
                }
                _ => {
                    if lower.starts_with("mouse") {
                        return Err(AppError::Hotkey(format!(
                            "不支持的鼠标按键：{token}，请使用 MouseLeft/MouseRight/MouseMiddle/MouseBack/MouseForward"
                        )));
                    }
                    return Err(AppError::Hotkey(format!("不支持的快捷键字段：{token}")));
                }
            }
        }

        if !has_button {
            return Err(AppError::Hotkey(
                "鼠标快捷键必须包含 Mouse* 主键".to_string(),
            ));
        }

        Ok(binding)
    }

    fn extract_button(w_param: u32, l_param: LPARAM) -> Option<(MouseButton, bool)> {
        match w_param {
            WM_LBUTTONDOWN => Some((MouseButton::Left, true)),
            WM_LBUTTONUP => Some((MouseButton::Left, false)),
            WM_RBUTTONDOWN => Some((MouseButton::Right, true)),
            WM_RBUTTONUP => Some((MouseButton::Right, false)),
            WM_MBUTTONDOWN => Some((MouseButton::Middle, true)),
            WM_MBUTTONUP => Some((MouseButton::Middle, false)),
            WM_XBUTTONDOWN | WM_XBUTTONUP => {
                let info = unsafe { *(l_param as *const MSLLHOOKSTRUCT) };
                let xbutton = ((info.mouseData >> 16) & 0xffff) as u16;
                if xbutton == XBUTTON1 as u16 {
                    Some((MouseButton::Back, w_param == WM_XBUTTONDOWN))
                } else if xbutton == XBUTTON2 as u16 {
                    Some((MouseButton::Forward, w_param == WM_XBUTTONDOWN))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn key_is_down(vk: i32) -> bool {
        unsafe { (GetAsyncKeyState(vk) as u16 & 0x8000) != 0 }
    }

    fn modifiers_match(binding: &MouseBinding) -> bool {
        let ctrl = key_is_down(VK_CONTROL as i32);
        let alt = key_is_down(VK_MENU as i32);
        let shift = key_is_down(VK_SHIFT as i32);
        let meta = key_is_down(VK_LWIN as i32) || key_is_down(VK_RWIN as i32);

        binding.ctrl == ctrl && binding.alt == alt && binding.shift == shift && binding.meta == meta
    }

    unsafe extern "system" fn mouse_proc(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
        if n_code == HC_ACTION as i32 {
            let guard = hook_context().lock();
            if let Some(context) = guard.as_ref() {
                if let Some((button, is_pressed)) = extract_button(w_param as u32, l_param) {
                    if button == context.binding.button && modifiers_match(&context.binding) {
                        match context.mode {
                            GateMode::Ptt => {
                                if is_pressed {
                                    context.gate.set_open(true, "mouse_hotkey");
                                } else {
                                    context.gate.set_open(false, "mouse_hotkey");
                                }
                            }
                            GateMode::Toggle | GateMode::Hybrid => {
                                if is_pressed {
                                    context.gate.toggle("mouse_hotkey");
                                }
                            }
                        }

                        let _ = context
                            .app
                            .emit("gate_state_changed", context.gate.snapshot());
                    }
                }
            }
        }

        unsafe { CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param) }
    }

    fn run_hook_thread(context: HookContext, ready_tx: mpsc::SyncSender<Result<u32, String>>) {
        unsafe {
            *hook_context().lock() = Some(context);

            let thread_id = GetCurrentThreadId();
            let module_handle = GetModuleHandleW(std::ptr::null());
            let hook = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), module_handle, 0);

            if hook == std::ptr::null_mut() {
                let error = std::io::Error::last_os_error();
                *hook_context().lock() = None;
                let _ = ready_tx.send(Err(format!("安装鼠标全局 Hook 失败: {error}")));
                return;
            }

            let _ = ready_tx.send(Ok(thread_id));

            let mut msg = std::mem::zeroed();
            loop {
                let result = GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0);
                if result <= 0 {
                    break;
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            let _ = UnhookWindowsHookEx(hook);
            *hook_context().lock() = None;
        }
    }

    #[derive(Default)]
    pub struct MouseHookManager {
        worker: Mutex<Option<MouseHookWorker>>,
    }

    impl MouseHookManager {
        pub fn register(
            &self,
            app: &tauri::AppHandle,
            accelerator: &str,
            gate: std::sync::Arc<GateController>,
            mode: GateMode,
        ) -> Result<(), AppError> {
            self.unregister();

            let binding = parse_mouse_binding(accelerator)?;
            let context = HookContext {
                binding,
                gate,
                app: app.clone(),
                mode,
            };

            let (ready_tx, ready_rx) = mpsc::sync_channel::<Result<u32, String>>(1);
            let join_handle = thread::spawn(move || run_hook_thread(context, ready_tx));

            match ready_rx.recv_timeout(Duration::from_secs(3)) {
                Ok(Ok(thread_id)) => {
                    *self.worker.lock() = Some(MouseHookWorker {
                        thread_id,
                        join_handle: Some(join_handle),
                    });
                    Ok(())
                }
                Ok(Err(message)) => {
                    let _ = join_handle.join();
                    Err(AppError::Hotkey(message))
                }
                Err(_) => {
                    let _ = join_handle.join();
                    Err(AppError::Hotkey(
                        "注册鼠标全局 Hook 超时，请重试".to_string(),
                    ))
                }
            }
        }

        pub fn unregister(&self) {
            if let Some(mut worker) = self.worker.lock().take() {
                unsafe {
                    let _ = PostThreadMessageW(worker.thread_id, WM_QUIT, 0, 0);
                }
                if let Some(handle) = worker.join_handle.take() {
                    let _ = handle.join();
                }
            }
            *hook_context().lock() = None;
        }
    }

    impl Drop for MouseHookManager {
        fn drop(&mut self) {
            self.unregister();
        }
    }
}

#[cfg(target_os = "windows")]
pub use imp::MouseHookManager;

#[cfg(not(target_os = "windows"))]
#[derive(Default)]
pub struct MouseHookManager;

#[cfg(not(target_os = "windows"))]
impl MouseHookManager {
    pub fn register(
        &self,
        _app: &tauri::AppHandle,
        _accelerator: &str,
        _gate: Arc<GateController>,
        _mode: GateMode,
    ) -> Result<(), AppError> {
        Err(AppError::Hotkey("鼠标全局快捷键仅支持 Windows".to_string()))
    }

    pub fn unregister(&self) {}
}
