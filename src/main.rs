use crate::constants::{CSS, DEFAULT_JSON};
use crate::ui::build_ui;
use crate::utils::ConfFile;
use dirs::home_dir;
use gtk4::{self as gtk, CssProvider, Settings, gdk::Display, prelude::*};
use std::fs::{self, File};
use std::io;
use std::path::PathBuf;

mod constants;
mod events;
mod ui;
mod utils;

fn files_init() -> io::Result<()> {
    let home = home_dir().expect("Failed to determine home directory!");
    let void_launcher = home.join(".config/void-launcher");
    let conf = void_launcher.join("config.json");
    if !void_launcher.exists() || !void_launcher.is_dir() {
        println!("Creating {} directory.", void_launcher.to_string_lossy());
        if !home.join(".config").exists() {
            fs::create_dir_all(void_launcher)?;
        } else {
            fs::create_dir(void_launcher)?;
        }
    }

    if !conf.exists() || !conf.is_file() {
        println!("Creating {}", conf.to_string_lossy());
        File::create(conf.clone())?;
        fs::write(conf, DEFAULT_JSON)?;
    }

    Ok(())
}

pub fn is_dark_theme_active() -> bool {
    let mut theme = true;
    // let mut theme_name = String::new();
    if let Some(display) = Display::default() {
        let settings = Settings::for_display(&display);

        let theme_name = settings.gtk_theme_name().unwrap();

        if theme_name.to_lowercase().contains("light") {
            theme = false;
        }
    } else {
        println!("No default display found.");
    }

    theme
}
fn main() -> io::Result<()> {
    let application = gtk::Application::builder()
        .application_id("com.void-launcher.com")
        .build();
    files_init()?;

    application.connect_startup(|app| {
        let provider = CssProvider::new();
        let home = home_dir().expect("Failed to determine home directory!");
        let mut css = CSS.to_string();
        let css_path = home.join(".config/void-launcher/style.css");
        if css_path.exists() && css_path.is_file() {
            let new_css = ConfFile::new(css_path).expect("Failed to create ConfFile.");
            css = format!("{}\n{}", css, new_css.read());
        }

        provider.load_from_string(&css);

        gtk::style_context_add_provider_for_display(
            &Display::default().expect("Couldn't connect to display."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_USER,
        );

        build_ui(app).expect("Failed to build UI!");
    });

    application.run();

    Ok(())
}
