//! `org.freedesktop.a11y.KeyboardMonitor`:
//!
//! Interface for monitoring of keyboard input by assistive technologies.
//!
//! This interface is used by assistive technologies to monitor keyboard input of the compositor.
//! The compositor is expected to listen on the well-known bus name "org.freedesktop.a11y.Manager" at the object path "/org/freedesktop/a11y/Manager".
//!
//! ## Security
//!
//! This protocol, while relatively widely used, is commonly restricted to a hardcoded list of
//! clients, namely the `Orca` screen reader.
//! In order to use this library in development mode, you may need to add your applications's
//! [well-known bus name]() to the list of acceptable assistive technologies, and compile from
//! scratch.
//!
//! How to accomplish this will need to be found out for yourself, as there are too many
//! compositors to enumerate your options.
//!
//! If you fail to do this, the compositor is well within its rights to disregard all messages on this
//! bus without any further interaction.

pub mod state_machine;
#[cfg(test)]
mod test;

use serde::{Deserialize, Serialize};
use xkeysym::Keysym as InnerKeysym;
use zbus::proxy;
use zbus::zvariant::{Signature, Type};

use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Keysym(pub InnerKeysym);

impl Keysym {
    fn key_char(&self) -> Option<char> {
        self.0.key_char()
    }
}
impl From<InnerKeysym> for Keysym {
    fn from(iks: InnerKeysym) -> Self {
        Keysym(iks)
    }
}

impl Not for Keysym {
    type Output = Self;
    fn not(self) -> Self {
        Keysym(InnerKeysym::new(!self.0.raw()))
    }
}

impl Type for Keysym {
    const SIGNATURE: &'static Signature = u32::SIGNATURE;
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct ModMask(pub InnerKeysym);

impl ModMask {
    pub fn is_empty(&self) -> bool {
        self.0.raw() == 0
    }
    pub fn empty() -> Self {
        ModMask(InnerKeysym::from(0))
    }
}

impl BitAnd<Self> for ModMask {
    type Output = Self;
    fn bitand(self, rhs: Self) -> ModMask {
        ModMask((self.0.raw() & rhs.0.raw()).into())
    }
}
impl BitAnd<Keysym> for ModMask {
    type Output = Self;
    fn bitand(self, rhs: Keysym) -> ModMask {
        ModMask((self.0.raw() & rhs.0.raw()).into())
    }
}
impl BitOr<Keysym> for ModMask {
    type Output = ModMask;
    fn bitor(self, rhs: Keysym) -> ModMask {
        ModMask((self.0.raw() | rhs.0.raw()).into())
    }
}
impl BitOrAssign<Keysym> for ModMask {
    fn bitor_assign(&mut self, rhs: Keysym) {
        *self = ModMask((self.0.raw() | rhs.0.raw()).into())
    }
}
impl BitAndAssign<Keysym> for ModMask {
    fn bitand_assign(&mut self, rhs: Keysym) {
        *self = ModMask((self.0.raw() & rhs.0.raw()).into())
    }
}

impl Type for ModMask {
    const SIGNATURE: &'static Signature = u32::SIGNATURE;
}

#[proxy(
    interface = "org.freedesktop.a11y.KeyboardMonitor",
    default_path = "/org/freedesktop/a11y/Manager",
    default_service = "org.freedesktop.a11y.Manager"
)]
pub trait KeyboardMonitor {
    /// GrabKeyboard method
    fn grab_keyboard(&self) -> zbus::Result<()>;

    /// SetKeyGrabs method
    fn set_key_grabs(
        &self,
        modifiers: &[Keysym],
        keystrokes: &[&(Keysym, ModMask)],
    ) -> zbus::Result<()>;

    /// UngrabKeyboard method
    fn ungrab_keyboard(&self) -> zbus::Result<()>;

    /// UnwatchKeyboard method
    fn unwatch_keyboard(&self) -> zbus::Result<()>;

    /// WatchKeyboard method
    fn watch_keyboard(&self) -> zbus::Result<()>;

    /// KeyEvent signal
    #[zbus(signal)]
    fn key_event(
        &self,
        released: bool,
        state: ModMask,
        keysym: Keysym,
        unichar: char,
        keycode: u16,
    ) -> zbus::Result<()>;
}
