#[macro_use]
extern crate gdnative as godot;

mod conv;
mod direction;
mod player;
mod inventory;
mod enemy;

fn setup_logger() -> Result<(), log::SetLoggerError> {
    let log_level = log::LevelFilter::Info;

    let colors = fern::colors::ColoredLevelConfig::new()
        .debug(fern::colors::Color::Magenta)
        .trace(fern::colors::Color::Blue)
        .info(fern::colors::Color::Green)
        .warn(fern::colors::Color::Yellow)
        .error(fern::colors::Color::Red);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                colors.color(record.level()),
                message
            ))
        })
        .level(log_level)
        .chain(std::io::stdout())
        .chain(fern::Output::call(|message| {
            use log::Level::*;
            let printing_message = message.args();
            match message.level() {
                Info | Debug | Trace => godot_print!("{}", printing_message),
                Error | Warn => godot_warn!("{}", printing_message),
            }
        }))
        .apply()?;
    Ok(())
}

fn init(handle: godot::init::InitHandle) {
    setup_logger().expect("Logging setup.");

    handle.add_class::<player::Player>();
    handle.add_class::<player::Projectile>();
    handle.add_class::<enemy::Simple>();
}

godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();
