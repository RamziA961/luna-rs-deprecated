use poise::async_trait;

pub(crate) trait Log {
    fn log(&self);
}

#[async_trait]
pub(crate) trait AsyncLog {
    async fn async_log(&self);
}

pub(crate) fn build_logger() -> env_logger::Builder {
    use env_logger::fmt::Color;

    let mut log_builder = env_logger::builder();

    if cfg!(debug_assertions) {
        log_builder
            .filter_module("poise", log::LevelFilter::Info)
            .filter_module("songbird", log::LevelFilter::Info)
            .filter_module(module_path!(), log::LevelFilter::Debug)
            .filter_level(log::LevelFilter::Error)
    } else {
        log_builder
            .filter_module(module_path!(), log::LevelFilter::Warn)
            .filter_level(log::LevelFilter::Error)
    };

    log_builder.format(|buf, record| {
        use chrono::Local;
        use std::io::Write;

        let timestamp = Local::now().format("[%Y-%m-%d %H:%M:%S]");
        let level = record.level();

        let level_color = match level {
            log::Level::Error => Color::Red,
            log::Level::Warn => Color::Yellow,
            log::Level::Info => Color::Green,
            log::Level::Debug => Color::Blue,
            log::Level::Trace => Color::Magenta,
        };

        let mut timestamp_sty = buf.style();
        timestamp_sty
            .set_bg(Color::Rgb(139, 0, 139))
            .set_color(Color::White);

        let mut level_sty = buf.style();
        level_sty
            .set_color(level_color)
            .set_intense(true)
            .set_bold(true);

        let mut mod_sty = buf.style();
        mod_sty.set_color(Color::Blue).set_dimmed(true);

        write!(
            buf,
            "{} |{}| {} @{}:{}\n{}\n\n",
            timestamp_sty.value(timestamp),
            level_sty.value(level),
            mod_sty.value(record.module_path().unwrap_or("unspecified mod")),
            record
                .file()
                .and_then(|p| p.rsplit('/').next())
                .unwrap_or("unspecified file"),
            record.line().unwrap_or(0),
            record.args()
        )
    });

    log_builder
}
