use gtk4::{self as gtk, prelude::*};
use libadwaita::{self as adw, prelude::*};

use std::rc::Rc;
use std::cell::Cell;

use crate::ui::*;

use super::preferences::PreferencesStack;
use super::ToastError;

use crate::lib::game;

/// This structure is used to describe widgets used in application
/// 
/// `AppWidgets::try_get` function loads UI file from `.assets/ui/.dist` folder and returns structure with references to its widgets
/// 
/// This function does not implement events
#[derive(Clone)]
pub struct AppWidgets {
    pub window: adw::ApplicationWindow,
    pub toast_overlay: adw::ToastOverlay,

    pub leaflet: adw::Leaflet,

    pub launch_game: adw::SplitButton,
    pub launch_game_debug: gtk::Button,
    pub open_preferences: gtk::Button,

    pub launch_game_group: adw::PreferencesGroup,
    pub progress_bar_group: adw::PreferencesGroup,
    pub progress_bar: gtk::ProgressBar,

    pub preferences_stack: PreferencesStack
}

impl AppWidgets {
    fn try_get() -> Result<Self, String> {
        let builder = gtk::Builder::from_string(include_str!("../../assets/ui/.dist/main.ui"));

        let window = get_object::<adw::ApplicationWindow>(&builder, "window")?;
        let toast_overlay = get_object::<adw::ToastOverlay>(&builder, "toast_overlay")?;

        let result = Self {
            window: window.clone(),
            toast_overlay: toast_overlay.clone(),

            leaflet: get_object(&builder, "leaflet")?,

            launch_game: get_object(&builder, "launch_game")?,
            launch_game_debug: get_object(&builder, "launch_game_debug")?,
            open_preferences: get_object(&builder, "open_preferences")?,

            launch_game_group: get_object(&builder, "launch_game_group")?,
            progress_bar_group: get_object(&builder, "progress_bar_group")?,
            progress_bar: get_object(&builder, "progress_bar")?,

            preferences_stack: PreferencesStack::new(window, toast_overlay)?
        };

        // Add preferences page to the leaflet
        result.leaflet.append(&result.preferences_stack.preferences).set_name(Some("preferences_page"));

        Ok(result)
    }
}

/// This enum is used to describe an action inside of this application
/// 
/// It may be helpful if you want to add the same event for several widgets, or call an action inside of another action
#[derive(Debug)]
pub enum Actions {
    OpenPreferencesPage,
    PreferencesGoBack,
    LaunchGame
}

/// This enum is used to store some of this application data
/// 
/// In this example we store a counter here to know what should we increment or decrement
/// 
/// This must implement `Default` trait
#[derive(Debug, Default)]
pub struct Values;

/// The main application structure
/// 
/// `Default` macro automatically calls `AppWidgets::default`, i.e. loads UI file and reference its widgets
/// 
/// `Rc<Cell<Values>>` means this:
/// - `Rc` addeds ability to reference the same value from various clones of the structure.
///   This will guarantee us that inner `Cell<Values>` is the same for all the `App::clone()` values
/// - `Cell` addeds inner mutability to its value, so we can mutate it even without mutable reference.
/// 
/// So we have a shared reference to some value that can be changed without mutable reference.
/// That's what we need and what we use in `App::update` method
#[derive(Clone)]
pub struct App {
    widgets: AppWidgets,
    values: Rc<Cell<Values>>
}

impl App {
    /// Create new application
    pub fn new(app: &gtk::Application) -> Result<Self, String> {
        let result = Self {
            widgets: AppWidgets::try_get()?,
            values: Default::default()
        }.init_events();

        // Bind app to the window
        result.widgets.window.set_application(Some(app));

        Ok(result)
    }

    /// Add default events and values to the widgets
    fn init_events(self) -> Self {
        // Open preferences page
        let self_copy = self.clone();

        self.widgets.open_preferences.connect_clicked(move |_| {
            self_copy.update(Actions::OpenPreferencesPage);
        });

        // Go back button for preferences page
        let self_copy = self.clone();

        self.widgets.preferences_stack.preferences_go_back.connect_clicked(move |_| {
            self_copy.update(Actions::PreferencesGoBack);
        });

        // Launch game
        let self_copy = self.clone();
        
        self.widgets.launch_game.connect_clicked(move |_| {
            self_copy.update(Actions::LaunchGame);
        });

        self
    }

    /// Update widgets state by calling some action
    pub fn update(&self, action: Actions) {
        let values = self.values.take();

        println!("[update] action: {:?}, counter: {:?}", &action, &values);

        match action {
            Actions::OpenPreferencesPage => {
                self.widgets.leaflet.set_visible_child_name("preferences_page");

                if let Err(err) = self.widgets.preferences_stack.update() {
                    self.toast_error("Failed to update preferences", err);
                }
            }

            Actions::PreferencesGoBack => {
                self.widgets.leaflet.navigate(adw::NavigationDirection::Back);
            }

            Actions::LaunchGame => {
                // Display toast message if the game is failed to run
                if let Err(err) = game::run(false) {
                    self.toast_error("Failed to run game", err);
                }
            }
        }

        self.values.set(values);
    }

    /// Show application window
    pub fn show(&self) {
        self.widgets.window.show();
    }
}

impl ToastError for App {
    fn get_toast_widgets(&self) -> (adw::ApplicationWindow, adw::ToastOverlay) {
        (self.widgets.window.clone(), self.widgets.toast_overlay.clone())
    }
}

/*
pub enum AppState {
    Launch,
    Progress {
        title: String,
        progress: f64
    }
}

pub fn update_state(&self, state: AppState) {
    match state {
        AppState::Launch => {
            self.launch_game_group.set_visible(true);
            self.progress_bar_group.set_visible(false);
        },
        AppState::Progress { title, progress } => {
            self.launch_game_group.set_visible(false);
            self.progress_bar_group.set_visible(true);

            self.progress_bar.set_text(Some(&title));
            self.progress_bar.set_fraction(progress);
        }
    }
}
*/
