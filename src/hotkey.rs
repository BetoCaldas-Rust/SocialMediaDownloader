use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager,
    hotkey::{Code, HotKey, Modifiers},
};

pub struct CaptureHotkey {
    manager: GlobalHotKeyManager,
    hotkey: HotKey,
}

impl CaptureHotkey {
    pub fn register() -> Option<Self> {
        let manager = GlobalHotKeyManager::new().ok()?;
        let hotkey = HotKey::new(
            Some(Modifiers::SUPER | Modifiers::SHIFT),
            Code::KeyX,
        );
        manager.register(hotkey).ok()?;
        Some(Self { manager, hotkey })
    }

    pub fn triggered(&self) -> bool {
        let mut hit = false;
        while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.id() == self.hotkey.id() {
                hit = true;
            }
        }
        hit
    }
}

impl Drop for CaptureHotkey {
    fn drop(&mut self) {
        let _ = self.manager.unregister(self.hotkey);
    }
}
