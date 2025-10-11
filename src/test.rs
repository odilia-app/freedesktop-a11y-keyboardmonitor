use crate::state_machine::{KeyEvent, KeyEventType, Keystroke, State};
use crate::{Keysym, ModMask};
use xkeysym::Keysym as XKeysym;

#[test]
fn test_global_standard_keybind() {
    let mut state = State::default();
    let global_mods = vec![Keysym(XKeysym::Caps_Lock)];
    state.modifiers = global_mods;
    state.has_client = true;
    let events = vec![
        (XKeysym::H, false),
        (XKeysym::H, true),
        (XKeysym::Caps_Lock, false),
        (XKeysym::H, false),
        (XKeysym::H, true),
        (XKeysym::Caps_Lock, true),
        (XKeysym::H, false),
        (XKeysym::H, true),
    ];
    let expected_results: Vec<KeyEventType> = vec![
        KeyEventType::ProcessNormally,
        KeyEventType::ProcessNormally,
        KeyEventType::SendToAT(KeyEvent::new(
            false,
            ModMask::empty(),
            XKeysym::Caps_Lock.into(),
            None,
            0,
        )),
        KeyEventType::SendToAT(KeyEvent::new(
            false,
            ModMask::empty(),
            XKeysym::H.into(),
            Some('H'),
            0,
        )),
        KeyEventType::SendToAT(KeyEvent::new(
            true,
            ModMask::empty(),
            XKeysym::H.into(),
            Some('H'),
            0,
        )),
        KeyEventType::SendToAT(KeyEvent::new(
            true,
            ModMask::empty(),
            XKeysym::Caps_Lock.into(),
            None,
            0,
        )),
        KeyEventType::ProcessNormally,
        KeyEventType::ProcessNormally,
    ];
    let mut results = Vec::new();
    for ev in events {
        results.push(state.process(Keysym(ev.0), ev.1));
    }
    assert_eq!(
        results, expected_results,
        "Expected results do not match the running of the state machine!"
    );
}

#[test]
fn test_global_non_standard_keybind() {
    let mut state = State::default();
    let global_mods = vec![Keysym(XKeysym::H)];
    state.modifiers = global_mods;
    state.has_client = true;
    let local_keys = vec![Keystroke {
        modifiers: ModMask::empty(),
        keysym: XKeysym::F.into(),
    }];
    state.keystrokes = local_keys;
    let events = vec![
        (XKeysym::F, false),
        (XKeysym::F, true),
        (XKeysym::H, false),
        (XKeysym::F, false),
        (XKeysym::F, true),
        (XKeysym::H, true),
        (XKeysym::F, false),
        (XKeysym::F, true),
    ];
    let expected_results: Vec<KeyEventType> = vec![
        KeyEventType::ProcessNormally,
        KeyEventType::ProcessNormally,
        KeyEventType::SendToAT(KeyEvent::new(
            false,
            ModMask::empty(),
            XKeysym::H.into(),
            Some('H'),
            0,
        )),
        KeyEventType::SendToAT(KeyEvent::new(
            false,
            ModMask::empty(),
            XKeysym::F.into(),
            Some('F'),
            0,
        )),
        KeyEventType::SendToAT(KeyEvent::new(
            true,
            ModMask::empty(),
            XKeysym::F.into(),
            Some('F'),
            0,
        )),
        KeyEventType::SendToAT(KeyEvent::new(
            true,
            ModMask::empty(),
            XKeysym::H.into(),
            Some('H'),
            0,
        )),
        KeyEventType::ProcessNormally,
        KeyEventType::ProcessNormally,
    ];
    let mut results = Vec::new();
    for ev in events {
        results.push(state.process(Keysym(ev.0), ev.1));
    }
    assert_eq!(
        expected_results, results,
        "Expected results do not match the running of the state machine!"
    );
}
