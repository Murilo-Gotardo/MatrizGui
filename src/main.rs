use iced::{Application, Settings, Size, window};
use crate::home::LocaleList;

mod home;
mod commands;
mod json_sender;

fn main() {
    LocaleList::run(Settings {
        window: window::Settings {
            size: Size::new(450.0, 700.0),
            ..window::Settings::default()
        },
        ..Settings::default()
    }).expect("Falha ao executar aplicativo");
    
}
