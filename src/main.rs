use fern::Dispatch;
use iced::{Application, Settings};
use log;
use std::env;

mod desk;
mod types;
mod utils;
#[cfg(target_os = "windows")]
mod win32;

fn setup_logging(file_path: String) -> Result<(), fern::InitError> {
    Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(fern::log_file(file_path)?)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let settings = Settings {
        window: iced::window::Settings {
            size: iced::Size::new(520.0, 360.0),
            resizable: false,
            ..Default::default()
        },
        ..Default::default()
    };

    #[cfg(target_os = "windows")]
    {
        if args.iter().any(|arg| arg == "-service") {
            //This l
            setup_logging("C:\\deskhub_service_output.log".to_string())
                .expect("Failed to configure service logging.");
            let result = win32::service_ctrl::service_dispatch();
            log::info!("service dispatch result: {}", result);
            return;
        }

        if args.iter().any(|arg| arg == "-main") {
            desk::DeskWindow::run(settings)
                .expect("An error occurred while running the application");
            return;
        }

        win32::GuideWindow::run(settings).expect("An error occurred while running the application");
    }
}
