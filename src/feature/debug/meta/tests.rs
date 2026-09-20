// `super::super` is `debug/mod.rs`: its plugin and its public API.
use bevy::prelude::*;

use super::super::*;

#[test]
fn empty_value_turns_everything_off() {
    let settings = DebugSettings::parse("");
    assert!(DebugChannel::ALL.iter().all(|c| !settings.is_on(*c)));
}

#[test]
fn all_turns_every_channel_on() {
    let settings = DebugSettings::parse("all");
    assert!(DebugChannel::ALL.iter().all(|c| settings.is_on(*c)));
}

#[test]
fn comma_list_turns_on_only_those_channels() {
    let settings = DebugSettings::parse(" Picking , cells,unknown");
    assert!(settings.is_on(DebugChannel::Picking));
    assert!(settings.is_on(DebugChannel::Cells));
    assert!(!settings.is_on(DebugChannel::Input));
}

#[test]
fn toggle_flips_a_channel() {
    let mut settings = DebugSettings::default();
    settings.toggle(DebugChannel::Input);
    assert!(settings.is_on(DebugChannel::Input));
    settings.toggle(DebugChannel::Input);
    assert!(!settings.is_on(DebugChannel::Input));
}

#[test]
fn every_channel_has_its_own_name_and_key() {
    let names: Vec<&str> = DebugChannel::ALL.iter().map(|c| c.name()).collect();
    let mut unique = names.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(names.len(), unique.len());

    let keys: Vec<KeyCode> = DebugChannel::ALL.iter().map(|c| c.key()).collect();
    let mut unique_keys = keys.clone();
    unique_keys.sort_unstable_by_key(|k| format!("{k:?}"));
    unique_keys.dedup();
    assert_eq!(keys.len(), unique_keys.len());
    assert!(DebugSettings::parse("gameloop").is_on(DebugChannel::GameLoop));
}
