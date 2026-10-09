# Bug: `broadcast_selection` panics with `copy_from_slice` length mismatch

## Symptom

On the web (wasm) build, the app crashes with:

```
panicked at flexplore/src/cursors.rs:148:10:
copy_from_slice: source slice length (1) does not match destination slice length (0)
Uncaught RuntimeError: unreachable executed
```

After the panic the entire app hangs. This happens the first time a non-empty
node selection is broadcast to remote peers (i.e. as soon as a user clicks a
node while connected to a room).

## Location

`flexplore/src/cursors.rs:138` — `broadcast_selection` system:

```rust
pub fn broadcast_selection(
    cfg: Res<FlexConfig>,
    net: Res<NetState>,
    socket: Option<ResMut<MatchboxSocket>>,
    mut last: Local<Vec<usize>>,          // starts empty: len 0
) {
    let current = cfg.selected();          // e.g. [1] after user selects a node
    if current == last.as_slice() {
        return;
    }
    last.clone_from_slice(current);        // <-- panics (line 148)
    let Some(mut socket) = socket else { return };
    broadcast_unreliable(
        &mut socket,
        &net.peers,
        &NetMsg::Ephemeral(Ephemeral::Selection {
            path: current.to_vec(),
        }),
    );
}
```

## Root cause

`Vec::clone_from_slice` requires the source and destination slices to have the
**same length**; it panics otherwise (`len_mismatch_fail`).

`last` is a `Local<Vec<usize>>` initialized to an empty vector (len 0). The
first time the local selection becomes non-empty (len >= 1), the check
`current == last.as_slice()` is false (contents differ), so
`last.clone_from_slice(current)` is reached with mismatched lengths and panics.

Because this is a `panic!` inside wasm, the wasm instance traps
(`unreachable executed`), which freezes the whole app.

## Fix

Use `Vec::clone_from` instead of `clone_from_slice`. `clone_from` reallocates
the destination to match the source length, which is the intended behaviour
(remember the latest selection so subsequent broadcasts only fire on change):

```rust
last.clone_from(current);
```

## Verification notes

- Panic trace confirms the origin: `cursors::broadcast_selection` →
  `copy_from_slice_impl::len_mismatch_fail`.
- Triggered only in networked sessions after selecting a node; local-only use
  (`socket == None`) never reaches the panic because the broadcast is skipped,
  but `clone_from_slice` runs *before* the socket check, so the bug fires
  regardless as soon as the selection changes while a socket resource exists.
