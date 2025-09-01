use crate::state_machine::{
	State, KeyEventType, KeyEvent,
};
use crate::{Keysym, ModMask};
use xkeysym::Keysym as XKeysym;

#[test]
fn test_standard_keybind() {
	let mut state = State::default();
	let global_mods = vec![Keysym(XKeysym::Caps_Lock)];
	state.modifiers = global_mods;
	state.has_client = true;
	let events = vec![
		(XKeysym::Caps_Lock, false),
		(XKeysym::H, false),
		(XKeysym::H, true),
		(XKeysym::Caps_Lock, true),
	];
	let expected_results: Vec<KeyEventType> = vec![
		KeyEventType::Swallow,
		KeyEventType::SendToAT(KeyEvent::new(
			false,
			ModMask::empty(),
			XKeysym::H.into(),
			Some('H'),
			123,
		)),
		KeyEventType::SendToAT(KeyEvent::new(
			true,
			ModMask::empty(),
			XKeysym::H.into(),
			Some('H'),
			123,
		)),
		KeyEventType::Swallow,
	];
	let mut results = Vec::new();
	for ev in events {
		results.push(state.process(Keysym(ev.0), ev.1));
	}
	assert_eq!(expected_results, results, "Expected results do not match the running of the state machine!");
}
