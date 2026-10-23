# Magna Urbis engine snapshot

Source: https://github.com/dunkean/magna-urbis
Revision: `2988cb0941ad855fd09876a9d2d0e396a5ff04c1`
License: GPL-3.0-only; see LICENSE.

This directory contains the original `rust/crates/core` and `rust/crates/random` manifests and source trees, including their embedded catalogue data and the house-admission catalogue required by the urban stages. The local workspace manifest contains only these two crates. Upstream sources are otherwise unmodified. Upstream test sources are retained as part of the dependency; application validation does not run them.

Application integration is isolated in `src/integrations/magna_urbis/`. No other application module may import this dependency or retain its types. Update the snapshot and adapter together when upgrading.
