# Workspace components

`SplitLayout` builds a native Iced pane grid. It borrows the existing state and
returns the native widget, preserving pane keys, resize capture and child trees.
`SplitModel` borrows an app-owned weighted tree; the shared geometry functions
compile it to an Iced configuration, calculate minimum widths and update split
shares. Keep models valid (nonempty branches, one positive finite weight per
child) and retain stable pane-key reconciliation and serialization in the app.
`DockEdge` performs neutral-center edge hit testing without imposing panel limits
or deciding what a drop means.

`DragHandle` marks any content with app-owned metadata and forwards its input
immediately. `DragRegions` collects visible bounds, full bounds and focused input
through native scroll translations and clips. It excludes disjoint nested
viewports. An optional hideable handle exposes `DragVisibility` to an operation;
no model state is hidden or destroyed.

`DragGesture` retains origin/lift state. Choose inclusive `advance` or strict `crossed` threshold comparison to match
the desired drag behavior.
`cancel_drag` handles focus loss, Escape and optional pointer departure.
`edge_scroll` computes a step on the caller's existing tick. The library creates
no additional timer. Apps own snapshots, allowed targets, commit/rollback,
hover-to-reorder behavior and file operations.

`DropIndicator` shares allocation-free insertion/drop-preview painting, including
antialias fringes and clipping. Its appearance is explicit; use existing highlight
animations separately when a completed drop needs a pulse.

`TabBar::overflow(identity, start, end)` composes the keyed tab strip, a native
horizontal scroll area and `TabViewport`. A changed selected tab/range or viewport
width reveals the selected tab. Otherwise manual scrolling is preserved. The
viewport is also usable separately. `TabSizing` calculates clamped widths with
explicit control reservation and spacing. `TabSurface` paints a rounded tab while
title and close controls remain independent. Routes, open/close/reorder commands,
tab content, scans and saved state stay app-owned.

`Toolbar` returns a native row for command groups and custom search/action slots.
`AdaptiveToolbar` measures an app-supplied full-layout probe before selecting the
compact layout. Content and existing tooltip/anchor behavior stay in app code.

`NavigationItem` composes icon/badge and optional caption surfaces.
`AppShell` composes sidebar/body with optional header/footer slots.
`NavigationDrawer` provides backdrop/foreground slots around the existing shared
horizontal reveal. The app closes its logical drawer immediately and owns
dismissal actions and route changes. Closing reveal content receives no input.
Pass the current motion policy; disabled policies snap without animation.
The default builders preserve native row/stack topology without extra state hosts.

Applications retain route and workspace models, persistence, domain-specific transactions,
drop restrictions, artwork and command groups. Use only the primitives your layout needs.
