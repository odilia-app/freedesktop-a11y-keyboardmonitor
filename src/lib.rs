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

use zbus::proxy;
use xkeysym::Keysym as InnerKeysym;
use serde::{Serialize, Deserialize};
use zbus::zvariant::{Type, Signature};

#[derive(Serialize, Deserialize, Debug)]
#[repr(transparent)]
pub struct Keysym(pub InnerKeysym);

impl Type for Keysym {
    const SIGNATURE: &'static Signature = u32::SIGNATURE;
}


#[derive(Serialize, Deserialize, Debug)]
#[repr(transparent)]
pub struct ModMask(pub InnerKeysym);

impl Type for ModMask {
    const SIGNATURE: &'static Signature = u32::SIGNATURE;
}

#[proxy(
    interface = "org.freedesktop.a11y.KeyboardMonitor",
    default_path = "/org/freedesktop/a11y/Manager",
    default_service = "org.freedesktop.a11y.Manager",
)]
pub trait KeyboardMonitor {
	/// GrabKeyboard method
	fn grab_keyboard(&self) -> zbus::Result<()>;

	/// SetKeyGrabs method
	fn set_key_grabs(&self, modifiers: &[Keysym], keystrokes: &[&(Keysym, ModMask)]) -> zbus::Result<()>;

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
