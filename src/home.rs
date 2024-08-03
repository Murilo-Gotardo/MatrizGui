use std::fs::File;
use std::io::Read;
use std::net::{SocketAddr, UdpSocket};
use std::str::FromStr;
use std::sync::{Arc, mpsc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

use iced::{Element, Length, Sandbox};
use iced::Alignment::Center;
use iced::widget::{Button, Column, Container, Image, Row, Space, Text, TextInput};
use iced::widget::image::Handle;
use serde::{Deserialize, Serialize};
use crate::commands;

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub(crate) struct LocaleList {
    pub(crate) locale_list: Vec<Locale>,
    #[serde(default = "default_handler")]
    #[serde(skip)]
    bulbs: Arc<Mutex<Vec<Handle>>>,
    #[serde(skip)]
    client_addr: String,
    #[serde(skip)]
    timer: usize,
    #[serde(skip)]
    receiver: Arc<Mutex<Option<mpsc::Receiver<()>>>>,
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
    MakeNewTimer(usize, SocketAddr)
}

impl Sandbox for LocaleList {
    type Message = Message;

    fn new() -> Self {
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

        Self {
            locale_list: local_list,
            bulbs: locale_bulbs,
            client_addr: "0.0.0.0:0".to_string(),
            timer: 15,
            receiver: Arc::new(Mutex::new(None))
        }
    }

    fn title(&self) -> String {
        String::from("Home")
    }

    fn update(&mut self, message: Message) -> () {
        let mut socket = UdpSocket::bind(("0.0.0.0", 11001)).unwrap();

        // TODO: tirar o connect daqui
        if SocketAddr::from_str(&self.client_addr).is_ok() && self.client_addr != "0.0.0.0:0" {
            socket.connect(self.client_addr.clone()).expect("Failed to connect");
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
                commands::set(&mut socket, action.clone(), locate, addr);
                change_bulb_state(&action.to_lowercase(), index);
            },
            Message::MakeGet(index, locate, addr) => {
                let action_received = commands::get(&mut socket, locate, addr);
                change_bulb_state(&action_received, index);
            },
            Message::MakeGetAll(addr) => {
                let socket = Arc::new(Mutex::new(socket));
                let result = commands::get_all(socket.lock().unwrap(), addr);
                
                for (i, locale) in result.locale_list.iter().enumerate() {
                    if let Some(locale) = result.locale_list.iter().find(|&r| r.locate == locale.locate) {
                        change_bulb_state(locale.status.as_str(), i);
                    }
                }
            },
            Message::MakeServerIp(ip) => {
                self.client_addr = ip;
            },
            Message::MakeNewTimer(time, addr) => {
                let result = Arc::new(Mutex::new(LocaleList::default()));
                let result_clone = Arc::clone(&result);
                let receiver_clone = Arc::clone(&self.receiver);
                let bulbs_clone = Arc::clone(&self.bulbs);
                let (tx, rx) = mpsc::channel();
                let socket = Arc::new(Mutex::new(socket));
                
                thread::spawn(move || {
                    loop {
                        sleep(Duration::from_secs(time as u64));
                        let mut result_lock = result_clone.lock().unwrap();
                        *result_lock = commands::get_all(socket.lock().unwrap(), addr);

                        tx.send(()).unwrap();
                    }
                });
                
                *self.receiver.lock().unwrap() = Some(rx);
                
                thread::spawn(move || {
                    loop {
                        if let Some(ref receiver) = *receiver_clone.lock().unwrap() {
                            if let Ok(()) = receiver.try_recv() {
                                let result = result.lock().unwrap();
                                let mut bulbs_lock = bulbs_clone.lock().unwrap();
                                for (i, locale) in result.locale_list.iter().enumerate() {
                                    if let Some(locale) = result.locale_list.iter().find(|&r| r.locate == locale.locate) {
                                        if locale.status.as_str() == "on" {
                                            bulbs_lock[i] = on_bulb();
                                        } else {
                                            bulbs_lock[i] = off_bulb();
                                        }
                                    }
                                    break;
                                }
                            }
                        }
                        sleep(Duration::from_millis(100));
                    }
                });
            }
        }
    }

    fn view(&self) -> Element<Self::Message> {
        let mut column = Column::new();

        let input = TextInput::new("Digite o IP do servidor e sua porta", &self.client_addr)
            .on_input(|value| Message::MakeServerIp(value))
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
                    .on_press(Message::MakeSet(index, String::from("On"), item.locate.clone(), self.client_addr.clone().parse().unwrap())))
                .push(Button::new(Text::new("Off"))
                    .on_press(Message::MakeSet(index, String::from("Off"), item.locate.clone(), self.client_addr.clone().parse().unwrap())))
                .push(Button::new(Text::new("Update"))
                    .on_press(Message::MakeGet(index, item.locate.clone(), self.client_addr.clone().parse().unwrap())))
                .padding(15);

            column = column.push(row);
        }

        // TODO: fazer o input salvar o timer apenas com a tecla enter
        let timer_input = TextInput::new("Digite o tempo entre atualizações", &*self.timer.to_string())
            .on_input(|value| Message::MakeNewTimer(value.parse::<usize>().unwrap(), self.client_addr.parse().unwrap()))
            .padding(15);

        let update_all_btn = Button::new(Text::new("Update All"))
            .on_press(Message::MakeGetAll(self.client_addr.clone().parse().unwrap()));

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
