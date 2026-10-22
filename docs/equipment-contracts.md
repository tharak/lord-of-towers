# Equipment and tower order contracts

`EquippedElements` has three optional physical-unit slots. Duplicate types are allowed. New pickups fill the first empty slot not reserved for an incoming exchange; otherwise they become cargo. Cargo remains a separate `ElementInventory`; permanent `ElementStats` are never physical units. Every minion is born empty-equipped with full base health.

## Derived attributes

Fire modifies damage, water movement speed, earth maximum health and air vision. Each attribute uses `base × [1 + .10 × matching_stat × matching_equipped_copies × (1 + .10 × void_stat × equipped_void_copies)]`. Void alone grants no direct bonus. Base values are damage 10, speed 4, health 30 and vision 15. Birth stats start at two per element plus counts of the five consumed recipe units. Changing equipment preserves current/max health ratio.

## Orders and reservations

Player input queues `PlayerCommand` values. Ownership, live entities, slots and availability are validated before mutation. Each minion can have one pending `EquipmentOrder` entity. Its `ElementInventory` holds the reserved unit, removed immediately from source stock. Reservations cannot be reused for recipes or other orders.

An order records recipient, source tower, source position and action. The minion travels within `delivery_radius` (default 3). Equip swaps the reserved unit into the chosen slot and returns the previous unit to tower stock. ReturnEquipment immediately removes the chosen slot into minion cargo, removing its bonus before combat. That cargo is credited to tower stock only on arrival and appears in the cargo crown during travel. Return cargo may accompany a gathered unit. DeliverCargo transfers carried units normally. Defense can interrupt travel; other enemies are shot only when already in range.

Changing a grid choice cancels the previous order before requesting the new one. Cancellation refunds reservation to a surviving source tower; already unequipped cargo remains carried and follows normal delivery. Source or recipient destruction drops it at the recorded source position. Minion death separately drops all cargo and equipped units. Empty reservations generate no deposit. Completed, cancelled and invalid orders despawn; restart clears all match entities and command/selection state.

## Production

Only NPC towers automatically equip empty slots on nearby allied minions, oldest units first, before automatic production. They never replace existing equipment automatically. NPC births consume the oldest five remaining stock units.

The player chooses exactly five available units in the tower panel. Confirmation validates live stock before consuming the oldest units matching the selected counts, preserving remaining FIFO order. Consumed counts update `ElementEconomy`; newborn birth stats use that recipe and equipment is empty.

For each type: deposits + cargo + stock + equipped + reserved + consumed = initial. Visual children store no resources. Compilation is the only validation; no tests or runtime checks are added.

The minion panel is exactly three rows of five element buttons. Stock availability enables each type in all rows; the equipped type is outlined and can always be returned to a living friendly tower, even if its stock is zero. No slot selector, status, delivery or cancellation rows are shown. Tower recipe controls remain separate.
