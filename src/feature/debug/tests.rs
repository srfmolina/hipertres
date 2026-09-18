use super::*;

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
