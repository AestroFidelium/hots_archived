use tracing_appender::rolling;
pub fn init_config() {
    let file_appender = rolling::daily("logs", "server.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    #[cfg(debug_assertions)]
    {
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;

        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(non_blocking)
            .with_ansi(false)
            .with_file(true)
            .with_line_number(true)
            .with_target(true); // показывает модуль

        let stdout_layer = tracing_subscriber::fmt::layer()
            .with_writer(std::io::stdout)
            .with_file(true)
            .with_line_number(true)
            .with_target(true);

        tracing_subscriber::registry()
            .with(file_layer)
            .with(stdout_layer)
            .with(tracing_subscriber::filter::LevelFilter::DEBUG)
            .init();
    }

    #[cfg(not(debug_assertions))]
    {
        let subscriber = FmtSubscriber::builder()
            .with_max_level(tracing::Level::TRACE)
            .with_writer(BoxMakeWriter::new(non_blocking))
            .with_file(true)
            .with_line_number(true)
            .with_target(true)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("failed to set tracing subscriber");
    }

    std::mem::forget(_guard);
}
