use crate::ui::get_object;
use eyre::{Ok, Result};
use gtk4::{
    self as gtk, ApplicationWindow, Builder, EventControllerKey, gdk::Key, glib, prelude::*,
};

pub fn events(app: gtk::Application, builder: Builder) -> Result<()> {
    let window: ApplicationWindow = get_object(&builder, "window")?;
    let key_controller = EventControllerKey::new();
    key_controller.connect_key_pressed(move |_controller, keyval, _keycode, _state| {
        if keyval == Key::Escape {
            app.quit();
        }
        glib::Propagation::Proceed
    });
    window.add_controller(key_controller);

    Ok(())
}
