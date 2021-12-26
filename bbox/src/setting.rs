pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    fern::Dispatch::new()
        // Perform allocation-free log formatting
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}",
                message
            ))
            // out.finish(format_args!(
            //     "{}[{}][{}] {}",
            //     chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
            //     record.target(),
            //     record.level(),
            //     message
            // ))
        })
        // Add blanket level filter -
        .level(log::LevelFilter::Debug)
        // - and per-module overrides
        .level_for("hyper", log::LevelFilter::Info)
        // Output to stdout, files, and other Dispatch configurations
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        // Apply globally
        .apply()?;
    Ok(())
}