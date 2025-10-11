//! `odilia-input-server-keyboard`
//!
//! Library to handle state mechanics for keyboard controll of the Odilia screen reader.
//! Uses the `evdev` kernel interface to interrupt keys as necessary;
//! this allows Odilia to work anywhere: X11, Wayland, and TTY.

#![deny(
    clippy::all,
    clippy::pedantic,
    missing_docs,
    clippy::perf,
    clippy::complexity,
    clippy::style,
    rustdoc::all,
    clippy::print_stdout,
    clippy::print_stderr
)]

use crate::{Keysym as Key, ModMask};

/// A keystroke struct represents a combination of modifiers and key to be pressed in order to
/// trigger a [`KeyEvent`] signal to the AT.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Keystroke {
    /// Mask of modifiers that must be simultaniously pressed in order for the `keysym` field to
    /// trigger a send over to the AT.
    pub modifiers: ModMask,
    /// The key which triggers a [`KeyEvent`] signal to be sent to the AT.
    pub keysym: Key,
}

/// The primary holder of state for all keybindings in the daemon.
#[derive(Debug, Default)]
pub struct State {
    /// When set to false, clear all other fields and reset.
    /// Without this flag active, you will always recieve `KeyEvent::ProcessNormally`.
    pub has_client: bool,
    /// When set to true, grab _all_ key presses and releases.
    pub grab_all: bool,
    /// Whet set to true, _notify all_ (but do not grab) key presses and releases.
    pub notify_all: bool,

    /// Set of modifiers that are used for unconditional grabbing;
    /// if any of these keys are pressed, all other events (until release of
    /// the given key) will be grabbed and given to the AT.
    ///
    /// ```text
    /// Each item in @modifiers is an XKB keysym. All keys in this list
    /// will be grabbed, and keys pressed while any of these keys are down
    /// will also be grabbed.
    /// ```
    pub modifiers: Vec<Key>,
    /// All modifiers in `modifiers` that have been pressed
    ///
    /// TODO: key repeat delay
    pub pressed_modifiers: ModMask,
    /// A list of keystrokes of which may be sent to the AT pending their activation.
    pub keystrokes: Vec<Keystroke>,
    /// A list of pressed keys made _after_ global activation;
    /// this is stored so that subsequent releases (which may potentially be released after
    /// activation) are swallowed.
    ///
    /// Otherwise applications (and the compositor) could receive key up events for keys that were
    /// never pressed in the first place.
    pub pressed: Vec<Key>,
}

/// A key event accepted by an on-bus AT.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct KeyEvent {
    /// If it was a release event
    release: bool,
    /// The modmask at the time the event was activated
    state: ModMask,
    /// The keysym for this event
    keysym: Key,
    /// Unicode character that would be typed by this action
    /// TODO: how to calculate?
    unichar: Option<char>,
    /// Raw (hardware dependent) keycode
    /// TODO: how to caluclate?
    keycode: u16,
}
#[cfg(test)]
impl KeyEvent {
    /// Create a new `KeyEvent`; restricted to `test` mode only!
    ///
    /// - `release`: whether the event is a release event or not (false for press event)
    /// - `state`: [`ModMask`] describing the state of the keyboard (modifiers/latches/etc.) use
    ///   `Default` impl for simple non-modified sates.
    /// - `keysym`: which key was pressed/released
    /// - `unichar`: if possible, provide the character which this key event would produce: `None`
    ///   for all `release`d keys, and modifiers.
    /// - `keycode`: raw system-dependent keycode (unused).
    #[must_use]
    pub fn new(
        release: bool,
        state: ModMask,
        keysym: Key,
        unichar: Option<char>,
        keycode: u16,
    ) -> Self {
        KeyEvent {
            release,
            state,
            keysym,
            unichar,
            keycode,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
/// The action te perform based on the state of the keyboard handler
pub enum KeyEventType {
    /// Swallow the event; do not pass to AT, nor process as part of the key handling of the
    /// compositor.
    Swallow,
    /// Process the event normally.
    ProcessNormally,
    /// Send the following key event to the AT; do _not_ process though the compositor.
    SendToAT(KeyEvent),
    /// Process event normally _and_ send to AT
    SendToATAndProcess(KeyEvent),
}

impl State {
    /// Process a single event, and produce an enum of behaviours for the compositor to implement.
    pub fn process(&mut self, key: Key, release: bool) -> KeyEventType {
        if !self.has_client {
            return KeyEventType::ProcessNormally;
        }
        let key_event_inner = KeyEvent {
            release,
            keysym: key,
            unichar: key.key_char(),
            keycode: 0,
            state: self.pressed_modifiers,
        };
        let key_event = KeyEventType::SendToAT(key_event_inner.clone());
        let is_mod_global = self.modifiers.contains(&key);
        let any_pressed_mods = !self.pressed_modifiers.is_empty();
        let is_already_pressed = self.pressed.contains(&key);
        let is_mod_local = self
            .keystrokes
            .iter()
            .any(|ks| ks.modifiers | key == ks.modifiers);
        if self.grab_all && is_mod_global && release {
            self.grab_all = false;
            return key_event;
        }
        if self.grab_all {
            return key_event;
        }
        if self.notify_all {
            return KeyEventType::SendToATAndProcess(key_event_inner);
        }
        match (
            is_mod_global,
            any_pressed_mods,
            is_already_pressed,
            is_mod_local,
            release,
        ) {
            // a global modifier has been pressed,
            // and there are no current mods pressed
            (true, _, _, _, false) => {
                // add key to mask
                //self.pressed_modifiers |= key;
                self.grab_all = true;
                key_event
            }
            // a global modifier has been released
            // and there it is currently pressed
            (true, _, _, _, true) => {
                // remove key from mask
                //self.pressed_modifiers &= !key;
                self.grab_all = false;
                key_event
            }
            // a key has been pressed (or released),
            // a global modifer is pressed, and
            // this is not a repeat key (i.e. it either is part of the pressed keys and is being
            // released, or it is not part of the pressed keys and is being pressed)
            (false, true, false, _, false) => {
                self.pressed.push(key);
                key_event
            }
            (false, true, true, _, true) => {
                self.pressed.retain(|k| *k != key);
                key_event
            }
            // repeat keys while global grab is on;
            // it is up to the AT how to deal with such events, but no modification of the state
            // will occur.
            //
            // i.e. the release is true while the item is not in the pressed keys, or the release
            // is false while the item is already in the list
            (false, false, false, false, _) => KeyEventType::ProcessNormally,
            (false, false, _, _, _)
            | (false, true, false, _, true)
            | (false, true, true, _, false) => key_event,
        }
    }
}
