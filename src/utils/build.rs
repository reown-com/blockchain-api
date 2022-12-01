#![allow(dead_code)]

const BUILD_VERSION: &str = env!("VERGEN_BUILD_SEMVER");
const BUILD_PROFILE: &str = env!("VERGEN_CARGO_PROFILE");
const BUILD_FEATURES: &str = env!("VERGEN_CARGO_FEATURES");

const GIT_SHA: &str = env!("VERGEN_GIT_SHA");
const GIT_TAG: &str = env!("VERGEN_GIT_SEMVER");

#[derive(Debug, Clone, Copy, Default)]
pub struct CompileInfo;

impl CompileInfo {
    pub const fn build(&self) -> BuildInfo {
        BuildInfo
    }

    pub const fn git(&self) -> GitInfo {
        GitInfo
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BuildInfo;

impl BuildInfo {
    pub const fn version(&self) -> &'static str {
        BUILD_VERSION
    }

    pub const fn profile(&self) -> &'static str {
        BUILD_PROFILE
    }

    pub const fn features(&self) -> &'static str {
        BUILD_FEATURES
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GitInfo;

impl GitInfo {
    pub const fn hash(&self) -> &'static str {
        GIT_SHA
    }

    pub fn short_hash(&self) -> &'static str {
        let hash = self.hash();

        let end = hash
            .char_indices()
            .nth(7)
            .map(|(n, _)| n)
            .unwrap_or_else(|| hash.len());

        &hash[..end]
    }

    pub const fn tag(&self) -> &'static str {
        GIT_TAG
    }
}
