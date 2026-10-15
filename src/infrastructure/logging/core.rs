use std::path::Path;
use tracing_appender::non_blocking::WorkerGuard;

pub fn initialize(log_dir: impl AsRef<Path>, date_format: &str) -> [WorkerGuard; 2] {
    let log_dir = log_dir.as_ref();
    std::fs::create_dir_all(log_dir).expect("create log directory failed");

    let log_file = log_dir.join(format!("{}.log", chrono::Local::now().format(date_format)));
    let (stdout_writer, stdout_guard) = tracing_appender::non_blocking(std::io::stdout());
    let file = fern::log_file(&log_file).expect("open log file failed");
    let (file_writer, file_guard) = tracing_appender::non_blocking(file);

    fern::Dispatch::new()
        .format(|out, message, record| {
            let file = record.file().unwrap_or("<unknown>");
            let line = record.line().unwrap_or(0);
            out.finish(format_args!(
                "[{}] [{}] [{}] [{}:{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                file,
                line,
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .level_for("gpui_idea::application", log::LevelFilter::Debug)
        .level_for("gpui_idea::infrastructure", log::LevelFilter::Debug)
        .level_for("gpui_idea::gui::workspace", log::LevelFilter::Debug)
        .level_for("gpui_idea::gui::workspace::ssh", log::LevelFilter::Debug)
        .level_for("gpui_idea::gui::workspace::sftp", log::LevelFilter::Debug)
        .level_for(
            "gpui_idea::data_context::terminal::update",
            log::LevelFilter::Debug,
        )
        .chain(fern::Output::writer(Box::new(stdout_writer), "\n"))
        .chain(fern::Output::writer(Box::new(file_writer), "\n"))
        .apply()
        .expect("init logger failed");

    super::bridge::initialize();

    let default_panic_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        log::error!(
            "application panic: {info}\n{}",
            std::backtrace::Backtrace::force_capture()
        );
        default_panic_hook(info);
    }));

    log::info!("init logger success: {}", log_file.display());
    [stdout_guard, file_guard]
}
