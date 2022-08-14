use crate::{BuildInfo, Config};

pub struct State {
    pub config: Config,
    pub build_info: BuildInfo,
}

build_info::build_info!(fn build_info);

pub fn new_state(config: Config) -> State {
    let build_info: &BuildInfo = build_info();

    State {
        config,
        build_info: build_info.clone(),
    }
}
