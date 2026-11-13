use smithay::backend::input::Keycode;
use smithay::utils::Point;

use super::*;

fn primary_seat_name(f: &mut Fixture) -> String {
    f.niri().seat.name().to_owned()
}

#[test]
fn no_agent_seats_by_default() {
    let mut f = Fixture::new();
    f.add_output(1, (1920, 1080));
    let id = f.add_client();
    f.roundtrip(id);

    assert!(f.niri().agent_seats.is_empty());
    let names: Vec<_> = f
        .client(id)
        .state
        .seats
        .values()
        .map(|seat| seat.name.clone())
        .collect();
    assert!(names.iter().all(|name| !name.starts_with("niri-agent-")));
}

#[test]
fn agent_seats_are_advertised_by_name() {
    let mut f = Fixture::new();
    f.add_output(1, (1920, 1080));
    f.niri().add_agent_seats(2);
    let id = f.add_client();
    f.roundtrip(id);

    let mut names: Vec<_> = f
        .client(id)
        .state
        .seats
        .values()
        .map(|seat| seat.name.clone())
        .collect();
    names.sort();
    let primary = primary_seat_name(&mut f);
    let mut expected = vec![
        primary,
        String::from("niri-agent-1"),
        String::from("niri-agent-2"),
    ];
    expected.sort();
    assert_eq!(names, expected);
}

#[test]
fn agent_virtual_pointer_leaves_primary_pointer_alone() {
    let mut f = Fixture::new();
    f.add_output(1, (1920, 1080));
    f.niri().add_agent_seats(2);
    let id = f.add_client();
    f.roundtrip(id);

    let primary_before = f.niri().seat.get_pointer().unwrap().current_location();

    let seat = f.client(id).state.seat_named("niri-agent-1");
    let pointer = f.client(id).state.virtual_pointer_for_seat(&seat);
    pointer.motion(0, 10., 20.);
    pointer.frame();
    f.roundtrip(id);

    let niri = f.niri();
    let agent1 = niri.agent_seats[0]
        .get_pointer()
        .unwrap()
        .current_location();
    let agent2 = niri.agent_seats[1]
        .get_pointer()
        .unwrap()
        .current_location();
    let primary = niri.seat.get_pointer().unwrap().current_location();

    assert_eq!(agent1, Point::from((10., 20.)));
    assert_eq!(agent2, Point::from((0., 0.)));
    assert_eq!(primary, primary_before);
}

#[test]
fn primary_virtual_pointer_still_moves_primary_pointer() {
    let mut f = Fixture::new();
    f.add_output(1, (1920, 1080));
    f.niri().add_agent_seats(1);
    let id = f.add_client();
    f.roundtrip(id);

    let primary_name = primary_seat_name(&mut f);
    let seat = f.client(id).state.seat_named(&primary_name);
    let pointer = f.client(id).state.virtual_pointer_for_seat(&seat);
    let before = f.niri().seat.get_pointer().unwrap().current_location();
    pointer.motion(0, 10., 20.);
    pointer.frame();
    f.roundtrip(id);

    let niri = f.niri();
    let primary = niri.seat.get_pointer().unwrap().current_location();
    let agent = niri.agent_seats[0]
        .get_pointer()
        .unwrap()
        .current_location();
    assert_eq!(primary, before + Point::from((10., 20.)));
    assert_eq!(agent, Point::from((0., 0.)));
}

#[test]
fn agent_virtual_keyboard_leaves_primary_keyboard_alone() {
    let mut f = Fixture::new();
    f.add_output(1, (1920, 1080));
    f.niri().add_agent_seats(1);
    let id = f.add_client();
    f.roundtrip(id);

    let state = f.niri_state();
    let primary_keyboard = state.niri.seat.get_keyboard().unwrap();
    let keymap =
        primary_keyboard.with_xkb_state(state, |ctx| ctx.xkb().lock().unwrap().keymap_as_string());

    let seat = f.client(id).state.seat_named("niri-agent-1");
    let keyboard = f.client(id).state.virtual_keyboard_for_seat(&seat, &keymap);
    // KEY_A, pressed, then shift as a depressed modifier.
    keyboard.key(0, 30, 1);
    keyboard.modifiers(1, 0, 0, 0);
    f.roundtrip(id);

    let niri = f.niri();
    let agent_keyboard = niri.agent_seats[0].get_keyboard().unwrap();
    let primary_keyboard = niri.seat.get_keyboard().unwrap();

    assert!(agent_keyboard
        .pressed_keys()
        .contains(&Keycode::from(30u32 + 8)));
    assert!(agent_keyboard.modifier_state().shift);

    assert!(primary_keyboard.pressed_keys().is_empty());
    assert!(!primary_keyboard.modifier_state().shift);
}
