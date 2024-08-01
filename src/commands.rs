use std::fs::File;
use std::io::{Read, Write};
use std::net::{SocketAddr, UdpSocket};
use serde_json::json;
use crate::home::{Locale, LocaleList};
use crate::json_sender::JsonSender;

pub fn set(socket: &mut UdpSocket, value: String, locate: String, addr: SocketAddr) {

    let data = json!({
        "locate": locate,
        "value": value.to_lowercase(),
        "command": "set"
    });

    JsonSender::send_json_to_server(socket, data, addr);
    let json = JsonSender::receive_json_from_server(socket);
    let locale: Locale = serde_json::from_str(&json).expect("JSON inválido");
    
    let mut file = File::open("src/local.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    
    let mut locales: LocaleList = serde_json::from_str(&data).expect("JSON inválido");
    update_json_with_received_locale(locale, &mut locales);
    
    let updated_data = serde_json::to_string_pretty(&locales).expect("Erro ao serializar JSON");
    let mut file = File::create("src/local.json").unwrap();
    file.write_all(updated_data.as_bytes()).unwrap();
}

pub fn get(socket: &mut UdpSocket, locate: String, addr: SocketAddr) {
    let data = json!({
        "locate": locate,
        "command": "get"
    });

    JsonSender::send_json_to_server(socket, data, addr);
    let json = JsonSender::receive_json_from_server(socket);
    
    println!("{}", json)
}

fn update_json_with_received_locale(received_locale : Locale, locales: &mut LocaleList) -> Locale {
    for locale in &mut locales.locale_list {
        if locale.locate == received_locale.locate {

            locale.status = match received_locale.status.as_str() {
                "on" => "on".parse().unwrap(),
                "off" => "off".parse().unwrap(),
                _ => "deu ruim".parse().unwrap()
            };

            return locale.clone();
        }
    }
    
    return Locale::default();
}