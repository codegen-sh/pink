use indicatif::MultiProgress;
use indicatif_log_bridge::LogWrapper;

pub fn get_multi_progress() -> MultiProgress {
    let logger =
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).build();
    let level = logger.filter();
    let multi_progress = MultiProgress::new();
    log::set_max_level(level);
    LogWrapper::new(multi_progress.clone(), logger);
    return multi_progress;
}
