# Installation Modes

`packwire-rs/src/installer.rs` implements the three modes declared by manifests:

- `here`: downloads and extracts a package into Packwire's managed directory.
- `command`: runs the official installer or command defined by the manifest.
- `site`: opens the official site for tools that must be installed through their vendor.

The installer validates paths, prevents archive traversal, records state, and creates the corresponding shims.
