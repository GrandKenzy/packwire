# Version Resolution Engine

The resolution engine is implemented in `packwire-rs/src/resolver.rs`. It queries official sources for `stable` channels and fixed versions, then updates manifest URLs and hashes before installation.

Built-in resolvers cover Python, Node.js, and GitHub releases among others. Resolution runs before downloading and does not modify the original local catalog.
