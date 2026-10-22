# Minion drag and animation-state contracts

## Input and movement

Only friendly minions belonging to the living player tower faction accept dragging. Left mouse or one tracked touch starts a candidate; movement beyond eight logical screen pixels begins dragging. A short click/tap opens the existing panel on release. Dragging closes it.

`DragControl { target, phase }` is the authoritative player movement intent. Held targets follow the pointer's ground intersection and clamp to map bounds. `DragPhase::Held` moves the real minion without teleporting; `Released` finishes approaching the last pointer target. Each fixed step travels at most current upgraded speed × simulation delta, stopping exactly at the target without separation steering. A stationary pointer can be caught up to. When released and within one world unit, the component is removed before autonomous targeting resumes. Another drag may replace an old released target.

Drag movement overrides defense, shooting, enemy pursuit, gathering and equipment navigation/transfers. Pending orders, reservations, equipped units and cargo remain intact; existing projectiles still resolve, incoming damage applies, and cargo trails continue. Starting a drag clears the old shooting pause. Pause freezes movement but input may update target/phase.

A second finger cancels the held gesture and starts camera pan/pinch, suppressing new dragging until all fingers lift. Touch cancellation, lost focus, hidden tab, release outside the canvas or over UI cancel held control at the current position. Invalid/dead minions and restart clear the gesture. Completed released orders are not unrelated held-pointer state and are not cancelled by later camera gestures.

## Animation-facing components

Every starting/newborn minion has independent `LegState` and `ArmState` components. These are data for future animations, not mesh assets.

| LegState | Meaning |
|---|---|
| Idle | Normal behavior with no actual movement, including a shooting pause. |
| Moving | Actual displacement under normal autonomous behavior. |
| DragStart | Entire held drag, including waiting under the pointer. |
| DragRelease | Traveling to the last pointer position after release. |

`ArmState { carrying, shooting }` uses independent booleans: both can be true. Carrying derives from actual cargo (including returning equipment), not equipped slots. Shooting derives from the existing one-second shooting action. Idle arms means both flags are false. Fixed-step reconciliation follows combat and resource transfers; neither arm activity hides the other. The inspector shows leg/arm state and drag target/phase.

No new gameplay entities or resource units are created by dragging. Match restart replaces minions and their state/components. Validation is compilation only; no automated tests or runtime checks are added.
