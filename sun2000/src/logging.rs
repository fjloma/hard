use simplelog::*;
use std::fs::OpenOptions;

pub fn init(log_path: Option<String>) {
    let conf = ConfigBuilder::new()
        .set_time_format("%F, %H:%M:%S%.3f".to_string())
        .set_write_log_enable_colors(true)
        .build();

    let mut loggers = vec![];

    let console_logger: Box<dyn SharedLogger> = TermLogger::new(
        LevelFilter::Info,
        conf.clone(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    );
    loggers.push(console_logger);

    let mut logfile_error: Option<String> = None;
    match log_path {
        Some(ref log_path) => {
            let logfile = OpenOptions::new().create(true).append(true).open(log_path);
            match logfile {
                Ok(logfile) => {
                    loggers.push(WriteLogger::new(LevelFilter::Info, conf, logfile));
                }
                Err(e) => {
                    logfile_error = Some(format!(
                        "Error creating/opening log file: {:?}: {:?}",
                        log_path, e
                    ));
                }
            }
        }
        _ => {}
    };

    CombinedLogger::init(loggers).expect("Cannot initialize logging subsystem");
    if logfile_error.is_some() {
        error!("{}", logfile_error.unwrap());
        warn!("Will do console logging only...");
    }
}