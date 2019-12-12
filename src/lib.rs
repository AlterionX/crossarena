#[macro_use]
pub extern crate gdnative as godot;

mod entity;

mod event_bus;
mod records;

mod ui;

mod util;
mod systems;

mod crafting;

fn setup_logger() -> Result<(), log::SetLoggerError> {
    let log_level = log::LevelFilter::Info;

    let colors = fern::colors::ColoredLevelConfig::new()
        .debug(fern::colors::Color::Magenta)
        .trace(fern::colors::Color::Blue)
        .info(fern::colors::Color::Green)
        .warn(fern::colors::Color::Yellow)
        .error(fern::colors::Color::Red);

    let console_logger = fern::Dispatch::new()
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
        .chain(std::io::stdout());
    let godot_logger = fern::Dispatch::new()
        .format(move |out, message, record| {
            let line = record.line().map_or(-1, |l| l as i64);
            out.finish(format_args!(
                "{}[{}:{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                line,
                record.level(),
                message
            ))
        })
        .chain(fern::Output::call(|message| {
            use log::Level::*;
            let printing_message = message.args();
            match message.level() {
                Info | Debug | Trace => godot_print!("{}", printing_message),
                Warn => godot_warn!("{}", printing_message),
                Error => godot_error!("{}", printing_message),
            }
        }));

    fern::Dispatch::new()
        .level(log_level)
        .chain(console_logger)
        .chain(godot_logger)
        .apply()
}

fn init(handle: gdnative::init::InitHandle) {
    setup_logger().expect("Logging setup.");

    // Primary scene
    handle.add_class::<entity::Player>();
    handle.add_class::<entity::NormalProjectile>();
    handle.add_class::<entity::ChargedProjectile>();
    handle.add_class::<entity::MeleeAttack>();

    handle.add_class::<entity::SimpleEnemy>();
    handle.add_class::<entity::RangedEnemy>();

    handle.add_class::<entity::Arena>();
    handle.add_class::<entity::Switch>();
    handle.add_class::<entity::Forge>();

    // Screens
    handle.add_class::<ui::UI>();
    handle.add_class::<ui::HUD>();
    handle.add_class::<ui::Start>();
    handle.add_class::<ui::End>();
    handle.add_class::<ui::Inventory>();
    handle.add_class::<ui::Crafting>();

    handle.add_class::<event_bus::EventBus>();
    handle.add_class::<records::Records>();
}

godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();
