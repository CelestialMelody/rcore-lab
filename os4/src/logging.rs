use log::{self, Level, LevelFilter, Log, Metadata, Record};

/// A simple logger that prints all log messages
struct SimpleLogger;

// 为了实现自定义的日志记录器，我们需要实现 log::Log trait
impl Log for SimpleLogger {
    /// Returns true if the log level is enabled for the specified metadata.
    // TOFIX 这里没有做什么判断
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }
    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let color = match record.level() {
            Level::Error => 31, // 红色
            Level::Warn => 93,  // Bright Yellow
            Level::Info => 34,  // 蓝色
            Level::Debug => 32, // 绿色
            Level::Trace => 36, // 青色
        };
        println!(
            "\u{1B}[{}m[{:>5}] {}\u{1B}[0m",
            color,
            record.level(),
            record.args(),
        );
    }
    // TOFIX 这里没有做什么判断
    fn flush(&self) {}
}

pub fn init() {
    static LOGGER: SimpleLogger = SimpleLogger; // 初始化一个 SimpleLogger
    log::set_logger(&LOGGER).unwrap(); // 设置日志记录器
    log::set_max_level(match option_env!("LOG") {
        // 通过环境变量 RUST_LOG 来控制日志的输出级别
        Some(s) => match s {
            "ERROR" => LevelFilter::Error,
            "WARN" => LevelFilter::Warn,
            "INFO" => LevelFilter::Info,
            "DEBUG" => LevelFilter::Debug,
            "TRACE" => LevelFilter::Trace,
            _ => LevelFilter::Off,
        },
        None => LevelFilter::Off,
    });
}
