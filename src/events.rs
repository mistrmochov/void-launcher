use crate::ui::get_object;
use eyre::{Ok, Result};
use gtk4::{
    self as gtk, ApplicationWindow, Box, Builder, Button, Entry, EventControllerKey, FlowBox,
    Image, Label, Orientation,
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
    key_controller.connect_key_pressed(move |_controller, keyval, _keycode, _state| {
        if keyval == Key::Escape {
            app.quit();
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
