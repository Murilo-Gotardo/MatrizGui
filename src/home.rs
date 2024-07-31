use std::fs::File;
use std::io::Read;
use std::net::UdpSocket;

use iced::{Element, Length, Sandbox};
use iced::Alignment::End;
use iced::widget::{Button, Column, Container, Image, Row, Space, Text};
use iced::widget::image::Handle;
use serde::{Deserialize, Serialize};

use crate::commands;

#[derive(Default, Serialize, Deserialize, Debug)]
pub(crate) struct LocaleList {
    locale_list: Vec<Locale>,
    #[serde(default = "default_handler")]
    #[serde(skip)]
    bulbs: Vec<Handle>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Locale {
    locate: String,
    status: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    MakeSet(usize, String, String),
    MakeGet(String)
}

impl Sandbox for LocaleList {

    type Message = Message;

    fn new() -> Self {
        let mut file = File::open("src/local.json").unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();
        let locale_json: LocaleList = serde_json::from_str(&data).expect("JSON inválido");
        let mut local_list: Vec<Locale> = vec![];
        let mut locale_bulbs: Vec<Handle> = vec![];

        for locale in locale_json.locale_list {
            let bulb = if locale.status == "on" {
                on_bulb()
            } else {
                off_bulb()
            };

            locale_bulbs.push(bulb);
            local_list.push(locale);
        }

        Self {
            locale_list: local_list,
            bulbs: locale_bulbs
        }
    }

    fn title(&self) -> String {
        String::from("Home")
    }

    fn update(&mut self, message: Message) {
        let mut socket = UdpSocket::bind(("0.0.0.0", 11001)).unwrap();
        socket.connect("192.168.0.4:11000").expect("TODO: panic message");
        
        match message {
            Message::MakeSet(index, action, locate) => {
                if action.to_lowercase() == "on" {
                    commands::set(&mut socket, action, locate);
                    self.bulbs[index] = on_bulb();
                } else {
                    commands::set(&mut socket, action, locate);
                    self.bulbs[index] = off_bulb();
                }
            },
            Message::MakeGet(locate) => {
                commands::get(&mut socket, locate)
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let mut column = Column::new();
        
        for (index, item) in self.locale_list.iter().enumerate() {

            let text = Text::new(&item.locate);
            let bulb = self.bulbs[index].clone();

            let row = Row::new()
                .spacing(12)
                .push(Image::new(bulb).width(20).height(20))
                .push(text)
                .push(Space::with_width(Length::Fill))
                .push(Button::new(
                    Text::new("On"),
                ).on_press(Message::MakeSet(index, String::from("On"), item.locate.clone())))
                .push(Button::new(
                    Text::new("Off"),
                ).on_press(Message::MakeSet(index, String::from("Off"), item.locate.clone())))
                .push(Button::new(
                    Text::new("Update"),
                ).on_press(Message::MakeGet(item.locate.clone())))
                .padding(15);

            column = column.push(row);
        }

        column = column.push(Button::new(Text::new("Update All"))).align_items(End);

        Container::new(column)
            .padding(10)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn off_bulb() -> Handle {
    Handle::from_path("src/img/off_bulb.png")
}

fn on_bulb() -> Handle {
    Handle::from_path("src/img/on_bulb.png")
}

fn default_handler() -> Vec<Handle> {
    let handlers: Vec<Handle> = vec![];

    return handlers
}
