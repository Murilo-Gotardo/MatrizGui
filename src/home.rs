use std::fs::File;
use std::io::Read;
use std::net::UdpSocket;
use iced::{Element, Length, Sandbox};
use iced::widget::{Button, Column, Container, Image, Row, Space, Text};
use iced::widget::image::Handle;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

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
    MakeSet(usize, String),
    MakeGet
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
        match message {
            Message::MakeSet(index, action) => {
                if action.to_lowercase() == "on" {
                    self.bulbs[index] = on_bulb();

                    let data = json!({
                        "locate": "aaaaaaa",
                        "status": action.to_lowercase(),
                        "command": "set"
                    });

                    let mut socket = UdpSocket::bind(("0.0.0.0", 11001)).unwrap();
                    socket.connect("192.168.0.4:11000").expect("TODO: panic message");

                    send_json_to_server(&mut socket, data);

                } else {
                    self.bulbs[index] = off_bulb();
                }
            },
            Message::MakeGet => {
                println!("Fazendo get...");
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
                ).on_press(Message::MakeSet(index, String::from("On"))))
                .push(Button::new(
                    Text::new("Off"),
                ).on_press(Message::MakeSet(index, String::from("Off"))))
                .push(Button::new(
                    Text::new("Update"),
                ).on_press(Message::MakeGet))
                .padding(15);

            column = column.push(row);
        }

        column = column.push(Button::new(Text::new("Update All")));

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

fn send_json_to_server(connection: &mut UdpSocket, data: Value) {
    let json_string = serde_json::to_string(&data).expect("Falha ao serializar JSON");
    let metadata = json_string.len() as u64;
    let bytes = json_string.as_bytes();
    
    connection.send(&metadata.to_le_bytes()).unwrap();
    connection.send(bytes).unwrap();
}
