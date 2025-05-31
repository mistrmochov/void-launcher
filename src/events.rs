use crate::ui::get_object;
use crate::utils::{ConfFile, get_conf_data};
use dirs::home_dir;
use eyre::{Ok, Result};
use gtk4::{
    self as gtk, ApplicationWindow, Box, Builder, Button, CssProvider, Entry, EventControllerKey,
    FlowBox, Image, Label, Orientation,
    gdk::Key,
    gio::{AppInfo, AppLaunchContext, DesktopAppInfo},
    glib,
    prelude::*,
};
use std::cell::RefCell;
use std::rc::Rc;

pub fn events(
    app: gtk::Application,
    builder: Builder,
    icon_size_memory: Rc<RefCell<i32>>,
) -> Result<()> {
    let window: ApplicationWindow = get_object(&builder, "window")?;
    let search_bar: Entry = get_object(&builder, "search-entry")?;
    let flowbox: FlowBox = get_object(&builder, "apps-box")?;
    let key_controller = EventControllerKey::new();
    let app_clone = app.clone();
    let flowbox_clone = flowbox.clone();
    let window_clone = window.clone();
    let builder_clone = builder.clone();
    key_controller.connect_key_pressed(move |_controller, keyval, _keycode, _state| {
        if keyval == Key::Escape {
            exit_animations(builder_clone.clone(), app.clone())
                .expect("Failed to execute exit animations!");
        } else if keyval == Key::Return || keyval == Key::KP_Enter {
            let selected = flowbox_clone.selected_children();
            if let Some(first_selected) = selected.first() {
                if let Some(button) = first_selected
                    .child()
                    .and_then(|w| w.downcast::<Button>().ok())
                {
                    unsafe {
                        if let Some(appynka) = button.data::<AppInfo>("app-info") {
                            window_clone.close();
                            let context = AppLaunchContext::new();
                            appynka
                                .read()
                                .launch(&[], Some(&context))
                                .unwrap_or_else(|err| {
                                    eprintln!("Failed to launch app: {}", err);
                                });
                            app.quit();
                        }
                    }
                }
            }
        }
        glib::Propagation::Proceed
    });
    window.add_controller(key_controller);

    let app_infos = AppInfo::all();

    search_bar.connect_changed(move |entry| {
        let query = entry.text().to_lowercase();
        let app_infos_clone = app_infos.clone();

        while let Some(child) = flowbox.first_child() {
            flowbox.remove(&child); // Use reference to child
        }

        for appynka in app_infos_clone {
            let name_low = appynka.name().to_lowercase();
            let mut matches = name_low.contains(&query);
            if let Some(desktop_info) = appynka.downcast_ref::<DesktopAppInfo>() {
                if let Some(keywords_str) = desktop_info.string("Keywords") {
                    let keywords: Vec<String> = keywords_str
                        .split(';')
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_lowercase())
                        .collect();
                    if keywords.iter().any(|k| k.contains(&query))
                        || appynka.name().to_lowercase().contains(&query)
                    {
                        if keywords.iter().any(|k| k.contains(&query)) {
                            matches = true;
                        }
                    }
                }
            }

            if matches && appynka.should_show() {
                let name = appynka.name();
                let err = format!("Failed to process the icon of app: {}.", name);
                let icon = appynka.icon().expect(&err);

                let label = Label::new(Some(&name));
                label.set_justify(gtk::Justification::Fill);
                label.add_css_class("app-label");
                label.set_ellipsize(pango::EllipsizeMode::End);
                label.set_max_width_chars(5); // or whatever works for your icon size
                label.set_wrap(false);
                let image = Image::from_gicon(&icon);

                let icon_size = *icon_size_memory.borrow();

                image.set_pixel_size(icon_size);
                image.set_valign(gtk4::Align::Center);
                image.set_halign(gtk4::Align::Center);
                let app_box = Box::new(Orientation::Vertical, 5);
                app_box.append(&image);
                app_box.append(&label);
                app_box.add_css_class("appynka");
                let app_button = Button::builder().child(&app_box).build();
                app_button.add_css_class("flat");
                app_button.add_css_class("app-button");
                unsafe {
                    app_button.set_data("app-info", appynka.clone());
                }

                flowbox.insert(&app_button, -1);

                apps_events(appynka, app_button, builder.clone(), app_clone.clone())
                    .expect("Failed to run app_events function.");
            }
        }
    });

    Ok(())
}

pub fn apps_events(
    appynka: AppInfo,
    app_button: Button,
    builder: Builder,
    app: gtk::Application,
) -> Result<()> {
    let window: ApplicationWindow = get_object(&builder, "window")?;
    let flowbox: FlowBox = get_object(&builder, "apps-box")?;
    let app_button_clone = app_button.clone();
    app_button.connect_clicked(move |_| {
        flowbox.unselect_all();
        app_button_clone.add_css_class("selected-button");
        window.close();
        let context = AppLaunchContext::new();
        appynka.launch(&[], Some(&context)).unwrap_or_else(|err| {
            eprintln!("Failed to launch app: {}", err);
        });
        app.quit();
    });

    Ok(())
}

pub fn start_animations(
    mut css: String,
    fullscreen: String,
    provider: CssProvider,
    builder: Builder,
) -> Result<()> {
    let window: ApplicationWindow = get_object(&builder, "window")?;
    let home = home_dir().expect("Couldn't locate the home dir.");
    let conf = ConfFile::new(home.join(".config/void-launcher/config.json"))?;

    let mut start_animation_mode = get_conf_data(conf.read(), "start-animation");
    let outline_box: Box =
        get_object(&builder, "outline-box").expect("Failed to get object: \"outline-box\"");
    let from = "transform: translateY(1px);";

    if (start_animation_mode != "true") && (start_animation_mode != "false") {
        println!(
            "\"{}\" isn't a valid option for start-animation, going with default: \"true\".",
            start_animation_mode
        );
        start_animation_mode = "true".to_string();
    }
    if (fullscreen == "false") && (start_animation_mode == "true") {
        if css.contains(from) {
            let to = format!("transform: translateY({}px);", window.height());
            css = css.replace(from, &to);
            provider.load_from_string(&css);
            window.set_opacity(1.0);
            glib::timeout_add_local_once(std::time::Duration::from_millis(20), move || {
                outline_box.add_css_class("outline-box-anim");
            });
        }
    } else if (fullscreen == "true") && (start_animation_mode == "true") {
        if css.contains(from) {
            let to = format!(
                "transform: translateY({}px);",
                (window.height() - ((window.height() as f64) * 0.70) as i32)
            );
            css = css.replace(from, &to);
            provider.load_from_string(&css);
            let mut opacity = 0.0;
            glib::timeout_add_local_once(std::time::Duration::from_millis(20), move || {
                glib::timeout_add_local(std::time::Duration::from_micros(1750), move || {
                    if opacity == 0.0 {
                        outline_box.add_css_class("outline-box-anim");
                    }
                    opacity += 0.0035;
                    window.set_opacity(opacity);
                    if opacity >= 1.0 {
                        if window.opacity() != 1.0 {
                            window.set_opacity(1.0);
                        }
                        glib::ControlFlow::Break
                    } else {
                        glib::ControlFlow::Continue
                    }
                });
            });
        }
    } else {
        let from = "transform: translateY(1px);";
        if css.contains(from) {
            let to = format!("transform: translateY({}px);", window.height());
            css = css.replace(from, &to);
            outline_box.add_css_class("outline-box-anim");
            provider.load_from_string(&css);
            window.set_opacity(1.0);
        }
    }
    Ok(())
}

fn exit_animations(builder: Builder, app: gtk::Application) -> Result<()> {
    let window: ApplicationWindow = get_object(&builder, "window")?;
    let home = home_dir().expect("Couldn't locate the home dir.");
    let conf = ConfFile::new(home.join(".config/void-launcher/config.json"))?;

    let mut exit_animation_mode = get_conf_data(conf.read(), "exit-animation");
    let mut fullscreen = get_conf_data(conf.read(), "fullscreen");
    let outline_box: Box =
        get_object(&builder, "outline-box").expect("Failed to get object: \"outline-box\"");

    if (exit_animation_mode != "true") && (exit_animation_mode != "false") {
        println!(
            "\"{}\" isn't a valid option for exit-animation, going with default: \"true\".",
            exit_animation_mode
        );
        exit_animation_mode = "true".to_string();
    }
    if (fullscreen != "true") && (fullscreen != "false") {
        println!(
            "\"{}\" isn't a valid option for fullscreen, going with default: \"true\".",
            fullscreen
        );
        fullscreen = "true".to_string();
    }

    if (fullscreen == "false") && (exit_animation_mode == "true") {
        outline_box.add_css_class("outline-box-anim-exit");
        glib::timeout_add_local_once(std::time::Duration::from_millis(500), move || {
            app.quit();
        });
    } else if (fullscreen == "true") && (exit_animation_mode == "true") {
        let mut opacity = 1.0;
        glib::timeout_add_local(std::time::Duration::from_micros(1750), move || {
            if opacity == 1.0 {
                outline_box.add_css_class("outline-box-anim-exit");
            }
            opacity -= 0.0035;
            window.set_opacity(opacity);
            if opacity <= 0.0 {
                if window.opacity() != 0.0 {
                    window.set_opacity(0.0);
                }
                app.quit();
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });
    } else {
        app.quit();
    }
    Ok(())
}
