use adw::prelude::*;

use gtk::glib;
use gtk::glib::clone;

use std::path::PathBuf;

use wait_not_await::Await;

use crate::lib::config;
use crate::ui::*;

pub fn choose_dir(current_folder: String) -> Await<Option<String>> {
    let dialogue = rfd::FileDialog::new()
        .set_directory(current_folder);

    let (sender, receiver) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        sender.send(dialogue.pick_folder()).unwrap();
    });

    Await::new(move || {
        match receiver.recv() {
            Ok(Some(path)) => Some(path.to_string_lossy().to_string()),
            Ok(None) => None,
            Err(_) => None
        }
    })
}

#[derive(Clone)]
pub struct Page {
    pub window: gtk::Window,
    pub page: gtk::Box,

    pub runners_folder: adw::ActionRow,
    pub dxvk_folder: adw::ActionRow,
    pub prefix_folder: adw::ActionRow,
    pub game_folder: adw::ActionRow,
    pub patch_folder: adw::ActionRow,
    pub temp_folder: adw::ActionRow,

    pub continue_button: gtk::Button,
    pub exit_button: gtk::Button
}

impl Page {
    pub fn new(window: gtk::Window) -> anyhow::Result<Self> {
        let builder = gtk::Builder::from_resource("/org/app/ui/first_run/default_paths.ui");

        let result = Self {
            window,
            page: get_object(&builder, "page")?,

            runners_folder: get_object(&builder, "runners_folder")?,
            dxvk_folder: get_object(&builder, "dxvk_folder")?,
            prefix_folder: get_object(&builder, "prefix_folder")?,
            game_folder: get_object(&builder, "game_folder")?,
            patch_folder: get_object(&builder, "patch_folder")?,
            temp_folder: get_object(&builder, "temp_folder")?,

            continue_button: get_object(&builder, "continue_button")?,
            exit_button: get_object(&builder, "exit_button")?
        };

        let config = config::get()?;

        // Add paths to subtitles
        result.runners_folder.set_subtitle(config.game.wine.builds.to_str().unwrap());
        result.dxvk_folder.set_subtitle(config.game.dxvk.builds.to_str().unwrap());
        result.prefix_folder.set_subtitle(config.game.wine.prefix.to_str().unwrap());
        result.game_folder.set_subtitle(config.game.path.to_str().unwrap());
        result.patch_folder.set_subtitle(config.patch.path.to_str().unwrap());
        result.temp_folder.set_subtitle(&match config.launcher.temp {
            Some(temp) => temp.to_string_lossy().to_string(),
            None => String::from("/tmp")
        });

        // Connect path selection events
        result.connect_activated(&result.runners_folder);
        result.connect_activated(&result.dxvk_folder);
        result.connect_activated(&result.prefix_folder);
        result.connect_activated(&result.game_folder);
        result.connect_activated(&result.patch_folder);
        result.connect_activated(&result.temp_folder);

        Ok(result)
    }

    fn connect_activated(&self, row: &adw::ActionRow) {
        row.connect_activated(clone!(@strong self.window as window => move |row| {
            let (sender, receiver) = glib::MainContext::channel::<String>(glib::PRIORITY_DEFAULT);

            choose_dir(row.subtitle().unwrap().to_string()).then(move |path| {
                if let Some(path) = path {
                    sender.send(path.clone()).unwrap();
                }
            });

            let row = row.clone();

            receiver.attach(None, move |path| {
                row.set_subtitle(&path);

                glib::Continue(false)
            });
        }));
    }

    pub fn update_config(&self, mut config: config::Config) -> config::Config {
        config.game.wine.builds = PathBuf::from(self.runners_folder.subtitle().unwrap().to_string());
        config.game.dxvk.builds = PathBuf::from(self.dxvk_folder.subtitle().unwrap().to_string());
        config.game.wine.prefix = PathBuf::from(self.prefix_folder.subtitle().unwrap().to_string());
        config.game.path        = PathBuf::from(self.game_folder.subtitle().unwrap().to_string());
        config.patch.path       = PathBuf::from(self.patch_folder.subtitle().unwrap().to_string());
        config.launcher.temp    = Some(PathBuf::from(self.temp_folder.subtitle().unwrap().to_string()));

        config
    }
}
