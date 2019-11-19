#[macro_use]
pub extern crate gdnative as godot;

mod conv;
mod direction;
mod error;
mod systems;
mod event_bus;

mod health;
mod player;
mod enemy;

mod switch;
mod attack;
mod projectile;
mod inventory;
mod arena;

mod ui;

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
            let line = record.line().map_or(-1, |l| l as i64);
            out.finish(format_args!(
                "{}[{}:{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                line,
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
                Warn => godot_warn!("{}", printing_message),
                Error => godot_error!("{}", printing_message),
            }
        }))
        .apply()?;
    Ok(())
}

fn init(handle: gdnative::init::InitHandle) {
    setup_logger().expect("Logging setup.");

    handle.add_class::<player::Player>();
    handle.add_class::<projectile::Normal>();
    handle.add_class::<attack::Attack>();

    handle.add_class::<enemy::SimpleEnemy>();

    handle.add_class::<arena::Arena>();
    handle.add_class::<switch::Switch>();

    handle.add_class::<ui::HUD>();

    handle.add_class::<event_bus::EventBus>();
}

godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();