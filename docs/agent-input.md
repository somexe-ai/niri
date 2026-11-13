# Background agent input

niri can expose independent Wayland input seats for background computer-use
agents while preserving the physical/user seat.

## Seat model

Agent seats are opt-in to preserve compatibility with generic Wayland tools
that blindly bind the last advertised `wl_seat`. Start niri with, for example,
`NIRI_AGENT_SEATS=2` to advertise:

- `niri-agent-1`
- `niri-agent-2`

`NIRI_AGENT_SEATS` accepts `0..8`; the default is `0`.
Each agent seat owns a distinct Smithay pointer and keyboard handle, so pointer
location, pointer focus, keyboard focus and modifier delivery are independent
from the primary seat.

The agent seats intentionally do not drive niri key bindings, focus-follows-mouse,
hot corners, overview gestures, physical keyboard LEDs, the user's cursor
renderer, or layout/workspace activation.

## Standard virtual input protocols

The existing protocol globals remain the transport:

- `zwlr_virtual_pointer_manager_v1` / `zwlr_virtual_pointer_v1` version 2
- `zwp_virtual_keyboard_manager_v1` / `zwp_virtual_keyboard_v1` version 1

A virtual pointer or virtual keyboard created with one of the `niri-agent-*`
seats is dispatched through that seat instead of the primary input pipeline.
Relative motion, absolute motion, buttons, axis source/stop/discrete and frame
semantics remain the standard wlroots protocol semantics.

Virtual keyboard events for an agent seat go straight to that seat's own
keyboard handle, bypassing key bindings, keyboard grabs, the input method and
the primary seat's virtual keyboard keymap swapping. A virtual keyboard that
sends its own keymap makes the agent seat's keyboard switch to it; one that
sends `no_keymap` uses the keymap the agent seat started with.

## Background foreign-toplevel activation

`zwlr_foreign_toplevel_handle_v1.activate(seat)` now preserves the supplied
seat. With the primary seat it keeps the normal niri behavior. With an agent
seat it sets only that seat's keyboard focus and does not raise the window,
switch workspaces or change the user's layout focus.

For pointer delivery, the activation target is bound to the concrete
client-side `wl_seat` resource that issued the request. Subsequent virtual
pointer motion from that same resource is hit-tested directly against the
selected toplevel's tile, independently of the user's stacking order. This
allows an occluded target (and a target on a non-active workspace of the same
output) to receive pointer focus without visually raising or switching to it.
A different Wayland connection starts with no pointer target, so explicit
desktop-pointer operations do not inherit a stale window selection.

For clients that require activation read-back, niri sends an `Activated`
state acknowledgement only to the foreign-toplevel handle that requested the
isolated activation. Global foreign-toplevel state continues to describe the
primary user focus, avoiding false focus changes in panels and task switchers.

All agent focus is cleared when session locking begins. Agent pointer and
keyboard events are dropped and isolated activation is refused until the
session is fully unlocked.

## Agent cursors

Agent pointers are not rendered. Cursor shape requests from clients focused on
an agent seat are ignored, so they never change the user's cursor.

## Client compatibility

A client that wants an isolated lane should:

1. enumerate `wl_seat` globals and pick a `niri-agent-N` one by name,
2. activate its target through wlr-foreign-toplevel using that seat,
3. create `zwlr_virtual_pointer_v1` and `zwp_virtual_keyboard_v1` objects for
   that same seat.

Clients that simply bind the last advertised `wl_seat` (such as upstream
`wtype`) end up on the last agent seat when agent seats are enabled, since
they are advertised after the primary seat. Concurrent automation of different
windows can be spread across the available `niri-agent-N` seats to get
independent pointer location/focus lanes without changing the protocol.
