# Testing

Public tests describe shared-library behavior and need no application checkout,
private fixture or credential.

- data_controls.rs: selection, tables, trees and column resizing.
- workspace_interactions.rs: splits, docking, drag/drop, responsive toolbars and drawers.
- forms_and_feedback.rs: form composition, read-only behavior, selectors, progress and control state.
- components.rs, keyboard.rs, policy.rs and visual.rs: component contracts,
  focus, motion policy and theme behavior.
- Inline unit tests cover widget internals; the gallery tests cover presentation.

Run cargo test --all-features --all-targets --locked and
cargo test --all-features --doc --locked from this repository.
The public CI workflow runs only this repository.

Application-specific frozen themes and historical UI-profile comparisons belong
in a separate private harness. They are not a dependency or workspace member of
sk-iced. Keep them out of public source exports and public CI jobs.

The crate includes top-level tests/*.rs explicitly. Package exclusion does not
make a committed file private: check both the working tree and Git history before
publishing a repository. Use a clean source snapshot for a new public repository
when the existing history contains private test fixtures. Preserve applicable
copyright and license notices.
