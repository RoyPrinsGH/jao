//! Local configuration support for `jao`.
//!
//! This module is responsible for loading and initializing the user's config
//! under `~/.jao/`.
//!
//! In the default build, the config currently exists mainly to point at the
//! trust manifest location used by the `trust-manifest` feature. The layout is
//! intentionally small so it can evolve without forcing incompatible changes on
//! callers or older config files.

mod load;
pub(crate) mod models;
mod persistence;
#[allow(unused_imports)]
pub(crate) use load::load_or_init;
