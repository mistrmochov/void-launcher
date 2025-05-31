use crate::constants::{BACK, CSS, CSS_DARK, CSS_LIGHT, DEFAULT_JSON};
use crate::ui::build_ui;
use crate::utils::{ConfFile, get_conf_data};
use dirs::home_dir;
use gtk4::{self as gtk, CssProvider, Settings, gdk::Display, prelude::*};
use regex::Regex;
use std::fs::{self, File};
use std::io;

mod constants;
mod events;
mod ui;
mod utils;

fn is_valid_hex_color(color: &str) -> bool {
    let re = Regex::new(r"^#[0-9a-fA-F]{6}$").unwrap();
    re.is_match(color)
}

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
        let mut css_dark = CSS_DARK.to_string();
        let mut css_light = CSS_LIGHT.to_string();

        let conf = ConfFile::new(home.join(".config/void-launcher/config.json"))
            .expect("Failed to create ConfFile.");
        let custom_background = get_conf_data(conf.read(), "background-color");
        let custom_accent_color = get_conf_data(conf.read(), "accent-color");
        let custom_select_color = get_conf_data(conf.read(), "select-color");

        if custom_background != "default" {
            if is_valid_hex_color(&custom_background) {
                if let Ok(regex_from) = Regex::new(r"background: .*") {
                    let to = format!("background: {};", custom_background);
                    let new_contents = regex_from.replace(BACK, to);
                    css = format!("{}\n{}", css, new_contents);
                } else {
                    println!("Couldn't process the Regex!");
                }
            } else {
                println!(
                    "{} isn't valid color, going with default.",
                    custom_background
                );
            }
        }
        if custom_accent_color != "default" {
            if is_valid_hex_color(&custom_accent_color) {
                if let Ok(regex_from) = Regex::new(r"--accent-color: .*") {
                    let to = format!("--accent-color: {};", custom_accent_color);
                    css_dark = regex_from.replace(&css_dark, to.clone()).to_string();
                    css_light = regex_from.replace(&css_light, to).to_string();
                } else {
                    println!("Couldn't process the Regex!");
                }
            } else {
                println!(
                    "{} isn't valid color, going with default.",
                    custom_accent_color
                );
            }
        }
        if custom_select_color != "default" {
            if is_valid_hex_color(&custom_select_color) {
                if let Ok(regex_from) = Regex::new(r"--select-color: .*") {
                    let to = format!("--select-color: {};", custom_select_color);
                    css_dark = regex_from.replace(&css_dark, to.clone()).to_string();
                    css_light = regex_from.replace(&css_light, to).to_string();
                } else {
                    println!("Couldn't process the Regex!");
                }
            } else {
                println!(
                    "{} isn't valid color, going with default.",
                    custom_select_color
                );
            }
        }

        if css_path.exists() && css_path.is_file() {
            let new_css = ConfFile::new(css_path).expect("Failed to create ConfFile.");
            css = format!("{}\n{}", css, new_css.read());
        }

        if is_dark_theme_active() {
            css = format!("{}\n{}", css_dark, css);
        } else {
            css = format!("{}\n{}", css_light, css);
        }

        gtk::style_context_add_provider_for_display(
            &Display::default().expect("Couldn't connect to display."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_USER,
        );

        build_ui(app, css, provider).expect("Failed to build UI!");
    });

    application.run();

    Ok(())
}
