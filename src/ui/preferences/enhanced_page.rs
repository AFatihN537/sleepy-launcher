use gtk4::{self as gtk, prelude::*};
use libadwaita::{self as adw, prelude::*};

use crate::ui::get_object;

pub struct Page;

impl Page {
    pub fn get() -> Result<adw::PreferencesPage, String> {
        let builder = gtk::Builder::from_string(include_str!("../../../assets/ui/.dist/preferences_enhanced.ui"));

        Ok(get_object(&builder, "enhanced_page")?)
    }

    pub fn title() -> String {
        String::from("Enhanced")
    }
}
