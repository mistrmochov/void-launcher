use std::path::PathBuf;

use crate::constants::UI_XML;
use crate::events::events;
use crate::is_dark_theme_active;
use crate::utils::{ConfFile, get_conf_data, string_to_i32};
use dirs::home_dir;
use eyre::{Ok, Result, eyre};
use gtk4::{
    self as gtk, ApplicationWindow, Box, Builder, Button, FlowBox, Image, Label, Orientation,
    Popover, gdk,
    gio::AppInfo,
    glib::{self, object::IsA},
    prelude::*,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::time::Duration;

pub fn build_ui(app: &gtk::Application) -> Result<()> {
    let builder = Builder::from_string(UI_XML);

    let window: ApplicationWindow = get_object(&builder, "window")?;
    let search_image: Image = get_object(&builder, "search-image")?;
    let flowbox: FlowBox = get_object(&builder, "apps-box")?;

    window.set_application(Some(app));
    if let Some(home) = home_dir() {
        let conf = ConfFile::new(home.join(".config/void-launcher/config.json"))?;
        let layer = get_conf_data(conf.read(), "layer");
        let fullscreen = get_conf_data(conf.read(), "fullscreen");
        let input_mode = get_conf_data(conf.read(), "input");
        let mut width = string_to_i32(get_conf_data(conf.read(), "width"), "width");
        let mut height = string_to_i32(get_conf_data(conf.read(), "height"), "height");
        let test_box = Box::new(Orientation::Horizontal, 10);
        //flowbox.set_selection_mode(gtk::SelectionMode::);

        window.init_layer_shell();
        if fullscreen == "false" {
            window.set_anchor(Edge::Bottom, true);
            if width < 350 {
                println!("Width of the window is too small, 350 is the lowest allowed!");
                width = 350;
            }
            if height < 200 {
                println!("Height of the window is too small, 200 is the lowest allowed!");
                height = 200;
            }
            window.set_default_height(height);
            window.set_default_width(width);
        } else {
            window.set_anchor(Edge::Top, true);
            window.set_anchor(Edge::Left, true);
            window.set_anchor(Edge::Right, true);
            window.set_anchor(Edge::Bottom, true);
        }

        if layer == "overlay" {
            window.set_layer(Layer::Overlay);
        } else if layer == "bottom" {
            window.set_layer(Layer::Bottom);
        } else {
            window.set_layer(Layer::Top);
        }
        // window.set_exclusive_zone(-1);
        if input_mode == "exclusive" {
            window.set_keyboard_mode(KeyboardMode::Exclusive);
        } else {
            window.set_keyboard_mode(KeyboardMode::OnDemand);
        }

        let search_image_file;
        if is_dark_theme_active() {
            search_image_file = PathBuf::from("/usr/share/void-launcher/icons/search-light.png");
        } else {
            search_image_file = PathBuf::from("/usr/share/void-launcher/icons/search-dark.png");
        }

        if !search_image_file.exists() {
            println!("Program's icons weren't found, please reinstall the program.");
            app.quit();
        } else {
            search_image.set_from_file(Some(search_image_file));
        }

        flowbox.set_row_spacing(15);
        let app_infos = AppInfo::all();
        for appynka in app_infos {
            // Filter only apps with show-in UI
            if appynka.should_show() {
                let name = appynka.name();
                let err = format!("Failed to process the icon of app: {}.", name);
                let icon = appynka.icon().expect(&err);
                let exec = appynka.executable(); // or app.launch() later

                let label = Label::new(Some(&name));
                label.set_justify(gtk::Justification::Fill);
                label.add_css_class("app-label");
                label.set_ellipsize(pango::EllipsizeMode::End);
                label.set_max_width_chars(5); // or whatever works for your icon size
                label.set_wrap(false);
                let image = Image::from_gicon(&icon);
                image.set_icon_size(gtk4::IconSize::Large);
                image.set_valign(gtk4::Align::Center);
                image.set_halign(gtk4::Align::Center);
                let app_box = Box::new(Orientation::Vertical, 5);
                app_box.append(&image);
                app_box.append(&label);
                app_box.add_css_class("appynka");

                flowbox.insert(&app_box, -1);
            }
        }

        let window_clone = window.clone();
        glib::idle_add_local_once(move || {
            let columns = match window_clone.width() {
                0..=499 => 3,
                500..=799 => 5,
                800..=1099 => 6,
                1100..=1399 => 7,
                1400..=1699 => 8,
                1700..=1999 => 9,
                _ => 10,
            };

            flowbox.set_max_children_per_line(columns);
            flowbox.set_min_children_per_line(columns);
        });

        events(app.to_owned(), builder)?;

        app.connect_activate(move |_| {
            window.present();
            window.set_decorated(false);
        });
    }

    Ok(())
}

pub fn get_object<T>(builder: &Builder, name: &str) -> Result<T>
where
    T: IsA<gtk4::glib::Object>,
{
    builder.object(name).ok_or(eyre!(
        "Unable to get UI element {}, this likely means the XML was changed/corrupted.",
        name
    ))
}
