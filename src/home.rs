use std::fs::File;
use std::io::Read;
use std::net::{SocketAddr, UdpSocket};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

use iced::{Application, Command, Element, Length, Theme};
use iced::Alignment::Center;
use iced::widget::{Button, Column, Container, Image, Row, Space, Text, TextInput};
use iced::widget::image::Handle;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::commands;

lazy_static!{
    pub static ref SOCKET: UdpSocket = UdpSocket::bind(("0.0.0.0", 11001)).unwrap();
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub(crate) struct LocaleList {
    pub(crate) locale_list: Vec<Locale>,
    #[serde(default = "default_handler")]
    #[serde(skip)]
    bulbs: Arc<Mutex<Vec<Handle>>>,
    #[serde(skip)]
    client_addr: String,
    #[serde(skip)]
    concrete_addr: String,
    #[serde(skip)]
    timer: Arc<Mutex<usize>>,
}

impl IntoIterator for LocaleList {
    type Item = Locale;
    type IntoIter = std::vec::IntoIter<Locale>;

    fn into_iter(self) -> Self::IntoIter {
        self.locale_list.into_iter()
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Locale {
    pub(crate) locate: String,
    pub(crate) status: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    MakeSet(usize, String, String, SocketAddr),
    MakeGet(usize, String, SocketAddr),
    MakeGetAll(SocketAddr),
    MakeServerIp(String),
    MakeConcreteIp,
    MakeNewTimer(String, SocketAddr),
}

impl Application for LocaleList {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (LocaleList, Command<Message>) {
        let mut file = File::open("src/local.json").unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();
        let locale_json: LocaleList = serde_json::from_str(&data).expect("JSON inválido");
        let mut local_list: Vec<Locale> = vec![];
        let locale_bulbs: Arc<Mutex<Vec<Handle>>> = Arc::new(Mutex::new(vec![]));

        for locale in locale_json.locale_list {
            let bulb = if locale.status == "on" {
                on_bulb()
            } else {
                off_bulb()
            };

            locale_bulbs.lock().unwrap().push(bulb);
            local_list.push(locale);
        }
        
        let timer = Arc::new(Mutex::new(10));

        let app = LocaleList {
            locale_list: local_list,
            bulbs: locale_bulbs,
            client_addr: "0.0.0.0:0".to_string(),
            concrete_addr: "0.0.0.0:0".to_string(),
            timer
        };

        (app, Command::none())
    }

    fn title(&self) -> String {
        String::from("Home")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        
        if SocketAddr::from_str(&self.client_addr).is_ok() && self.client_addr != "0.0.0.0:0" {
            SOCKET.connect(self.client_addr.clone()).expect("Failed to connect");
        }

        let change_bulb_state = |action: &str, index: usize| {
            if action.to_lowercase() == "on" {
                self.bulbs.lock().unwrap()[index] = on_bulb();
            } else {
                self.bulbs.lock().unwrap()[index] = off_bulb();
            }
        };

        match message {
            Message::MakeSet(index, action, locate, addr) => {
                commands::set(&SOCKET, action.clone(), locate, addr);
                change_bulb_state(&action.to_lowercase(), index);

                Command::none()
            },
            Message::MakeGet(index, locate, addr) => {
                let action_received = commands::get(&SOCKET, locate, addr);
                change_bulb_state(&action_received, index);

                Command::none()
            },
            Message::MakeGetAll(addr) => {
                let result = commands::get_all(&SOCKET, addr);

                for (i, locale) in result.locale_list.iter().enumerate() {
                    if let Some(locale) = result.locale_list.iter().find(|&r| r.locate == locale.locate) {
                        change_bulb_state(locale.status.as_str(), i);
                    }
                }

                Command::none()
            },
            Message::MakeServerIp(ip) => {
                self.client_addr = ip;

                Command::none()
            },
            Message::MakeConcreteIp => {
                self.concrete_addr = self.client_addr.clone();
                
                Command::none()
            },
            Message::MakeNewTimer(time, addr) => {
                let time = match time.parse::<usize>() {
                    Ok(t) => {
                        *self.timer.lock().unwrap() = t;
                        t
                    } Err(_) => {
                        0
                    }
                };

                thread::spawn(move || {
                    loop {
                        sleep(Duration::from_secs(time as u64));
                        println!("aaaaaa");
                        commands::get_all(&SOCKET, addr);
                    }
                });
                
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Self::Message> {        
        let mut column = Column::new();

        let input = TextInput::new("Digite o IP do servidor e sua porta", &self.client_addr)
            .on_input(|value| Message::MakeServerIp(value))
            .on_submit(Message::MakeConcreteIp)
            .padding(15);

        column = column.push(input);

        for (index, item) in self.locale_list.iter().enumerate() {
            let text = Text::new(&item.locate);
            let bulb = self.bulbs.lock().unwrap()[index].clone();

            let row = Row::new()
                .spacing(12)
                .push(Image::new(bulb).width(20).height(20))
                .push(text)
                .push(Space::with_width(Length::Fill))
                .push(Button::new(Text::new("On"))
                    .on_press(Message::MakeSet(index, String::from("On"), item.locate.clone(), self.concrete_addr.clone().parse().unwrap())))
                .push(Button::new(Text::new("Off"))
                    .on_press(Message::MakeSet(index, String::from("Off"), item.locate.clone(), self.concrete_addr.clone().parse().unwrap())))
                .push(Button::new(Text::new("Update"))
                    .on_press(Message::MakeGet(index, item.locate.clone(), self.concrete_addr.clone().parse().unwrap())))
                .padding(15);

            column = column.push(row);
        }
        
        let timer_input = TextInput::new("Digite o tempo entre atualizações", &*self.timer.lock().unwrap().to_string())
            .on_input(|value| Message::MakeNewTimer(value, self.concrete_addr.clone().parse().unwrap()))
            .padding(15);

        let update_all_btn = Button::new(Text::new("Update All"))
            .on_press(Message::MakeGetAll(self.concrete_addr.clone().parse().unwrap()));

        let update_row = Row::new()
            .spacing(12)
            .push(timer_input)
            .push(update_all_btn)
            .align_items(Center)
            .padding(15);

        column = column.push(update_row);

        Container::new(column)
            .padding(10)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn off_bulb() -> Handle {
    let handle = Handle::from_path("src/img/off_bulb.png");
    handle
}

fn on_bulb() -> Handle {
    let handle = Handle::from_path("src/img/on_bulb.png");
    handle
}

fn default_handler() -> Arc<Mutex<Vec<Handle>>>  {
    let handlers: Vec<Handle> = vec![];

    Arc::new(Mutex::new(handlers))
}