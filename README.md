# Bevy Status Effects (Name Pending)

A highly experimental, relationship-based, status effect system for Bevy.

There are currently two attempts at this project:

1. The `master` branch, which leverages the existing component hooks to uphold invariants.
2. The `command` branch which uses a custom `add_effect` method for upholding invariants.

Both have their tradeoffs, but I am not satisfied by either of them, at least not in their current state.