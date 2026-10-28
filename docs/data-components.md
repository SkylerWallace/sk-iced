# Data components

`VirtualList` renders fixed-height, app-owned rows through a bounded cache. Give
each dataset a stable `identity`, retain selection by item key in the app, and
map visible indices to those keys in callbacks. `row_height` changes preserve the
top item; `reveal(Some((serial, index)))` runs once per request serial. Ordinary
scrolling stays local and does not publish messages or invalidate app layout.
The cache retains visible row trees within a cache window. Persistent editor or
item state belongs in the model, since offscreen row trees can be recycled.

`TreeView` uses the same viewport without another widget-tree layer. Supply a
flattened visible sequence and use `TreeRow` for indentation plus independent
disclosure/content controls. Loading, empty and error rows are app-owned content.
`tree_navigation` supplies arrow behavior; the app gates keyboard focus, resolves
stable parents and performs expansion or loading. No disk enumeration occurs in
sk-iced.

`DataTable` composes a header and body without adding a scroll owner. `TableRow`
returns a native Iced row; cell widths, alignment and content remain explicit.
Use the existing shared scroll builders for horizontal scrolling and native or
virtual bodies as appropriate. Variable-height rows need a native scroll area,
not the fixed-height viewport.

`TableHeader` adds sort, reorder and context callbacks around a native row.
Callbacks use visible indices; map them to stable column IDs. `HeaderResize`
reserves each divider hit area so resize cannot also sort. `ColumnResize`
captures a drag locally and publishes bounded widths immediately. Its drag origin
survives view rebuilds; release or window focus loss ends capture. `TableColumns`
preserves ordered keys and hidden positions; apps own labels, sort semantics,
mandatory-column rules and serialization.

`select` handles replace, toggle and range selection over an ordered slice of
stable IDs without replacing app-owned selection state. `adjacent` supplies
bounded one-row navigation; application focus and command routing stay local.

These components add no animation, timer or motion subscription. Scroll, selection,
expansion and resizing remain immediate under every motion policy. Keep existing animation wrappers outside the row content where needed.

Applications retain loading, domain-specific rows, sorting rules, drag/drop transactions,
variable-height result content and persistence.
