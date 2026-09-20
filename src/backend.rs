use crate::config::Button;
use anyhow::{Context, Result};
use enigo::{Coordinate, Direction, Enigo, Mouse, Settings};

pub trait MouseOutput {
    fn press(&mut self, button: Button) -> Result<()>;
    fn release(&mut self, button: Button) -> Result<()>;
    fn position(&self) -> Result<(i32, i32)>;
    fn move_to(&mut self, x: i32, y: i32) -> Result<()>;
}

pub struct NativeMouse(Enigo);

impl NativeMouse {
    pub fn new() -> Result<Self> {
        #[cfg(all(unix, not(target_os = "macos")))]
        if std::env::var_os("DISPLAY").is_none() {
            anyhow::bail!("X11 DISPLAY is unavailable. This version needs an X11 desktop session.");
        }
        let settings = Settings {
            linux_delay: 0,
            ..Settings::default()
        };
        Ok(Self(
            Enigo::new(&settings).context("Cannot initialize mouse input")?,
        ))
    }
}

fn native_button(button: Button) -> enigo::Button {
    match button {
        Button::Left => enigo::Button::Left,
        Button::Right => enigo::Button::Right,
        Button::Middle => enigo::Button::Middle,
    }
}

impl MouseOutput for NativeMouse {
    fn press(&mut self, button: Button) -> Result<()> {
        self.0.button(native_button(button), Direction::Press)?;
        Ok(())
    }

    fn release(&mut self, button: Button) -> Result<()> {
        self.0.button(native_button(button), Direction::Release)?;
        Ok(())
    }

    fn position(&self) -> Result<(i32, i32)> {
        Ok(self.0.location()?)
    }

    fn move_to(&mut self, x: i32, y: i32) -> Result<()> {
        self.0.move_mouse(x, y, Coordinate::Abs)?;
        Ok(())
    }
}

/// Releases even if a wait is cancelled, an output operation fails, or a panic unwinds.
pub struct HeldMouse<M: MouseOutput> {
    pub output: M,
    held: Option<Button>,
}

impl<M: MouseOutput> HeldMouse<M> {
    pub fn new(output: M) -> Self {
        Self { output, held: None }
    }

    pub fn press(&mut self, button: Button) -> Result<()> {
        // Record before injection: a failing backend may have partially delivered input.
        self.held = Some(button);
        self.output.press(button)
    }

    pub fn release(&mut self) -> Result<()> {
        if let Some(button) = self.held {
            self.output.release(button)?;
            self.held = None;
        }
        Ok(())
    }
}

impl<M: MouseOutput> Drop for HeldMouse<M> {
    fn drop(&mut self) {
        let _ = self.release();
    }
}
