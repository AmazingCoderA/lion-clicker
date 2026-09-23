use anyhow::{bail, Context, Result};
use global_hotkey::{hotkey::HotKey, GlobalHotKeyManager};
use std::time::{Duration, Instant};

pub const STOP_HOTKEY: &str = "Ctrl+Shift+F12";

pub fn parse_trigger(text: &str) -> Result<HotKey> {
    let key: HotKey = text
        .trim()
        .parse()
        .context("Invalid hotkey (example: F6 or Ctrl+Shift+F8)")?;
    if key == STOP_HOTKEY.parse::<HotKey>()? {
        bail!("Ctrl+Shift+F12 is reserved for Stop");
    }
    Ok(key)
}

pub struct Hotkeys {
    manager: GlobalHotKeyManager,
    pub trigger: HotKey,
    pub stop: HotKey,
}

impl Hotkeys {
    pub fn new(text: &str) -> Result<Self> {
        let trigger = parse_trigger(text)?;
        let stop = STOP_HOTKEY.parse()?;
        let manager = GlobalHotKeyManager::new()?;
        manager
            .register(stop)
            .context("Cannot register stop hotkey")?;
        if let Err(error) = manager.register(trigger) {
            let _ = manager.unregister(stop);
            return Err(error).context("Cannot register trigger hotkey (already in use?)");
        }
        Ok(Self {
            manager,
            trigger,
            stop,
        })
    }

    pub fn rebind(&mut self, text: &str) -> Result<()> {
        let next = parse_trigger(text)?;
        if next == self.trigger {
            return Ok(());
        }
        // Register first: a conflict must not disable the previous working binding.
        self.manager
            .register(next)
            .context("Hotkey is unavailable")?;
        if let Err(error) = self.manager.unregister(self.trigger) {
            let _ = self.manager.unregister(next);
            return Err(error.into());
        }
        self.trigger = next;
        Ok(())
    }
}

impl Drop for Hotkeys {
    fn drop(&mut self) {
        let _ = self.manager.unregister(self.trigger);
        let _ = self.manager.unregister(self.stop);
    }
}

#[derive(Default)]
pub struct TapTracker {
    down: bool,
    count: u8,
    last: Option<Instant>,
}

impl TapTracker {
    pub fn event(&mut self, pressed: bool, now: Instant, required: u8, timeout: Duration) -> bool {
        if !pressed {
            self.down = false;
            return false;
        }
        if self.down {
            return false;
        }
        self.down = true;
        if self
            .last
            .is_none_or(|last| now.duration_since(last) > timeout)
        {
            self.count = 0;
        }
        self.last = Some(now);
        self.count += 1;
        if self.count >= required {
            self.count = 0;
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeat_is_ignored_and_expired_taps_do_not_accumulate() {
        let mut taps = TapTracker::default();
        let now = Instant::now();
        let timeout = Duration::from_millis(400);
        assert!(!taps.event(true, now, 2, timeout));
        assert!(!taps.event(true, now, 2, timeout));
        taps.event(false, now, 2, timeout);
        let later = now + Duration::from_millis(401);
        assert!(!taps.event(true, later, 2, timeout));
        taps.event(false, later, 2, timeout);
        assert!(taps.event(true, later + Duration::from_millis(30), 2, timeout));
    }
}
