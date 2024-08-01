use std::fs::File;
use std::io::Write;
use std::net::UdpSocket;
use serde_json::json;
use crate::json_sender::JsonSender;

pub fn set(socket: &mut UdpSocket, value: String, locate: String) {
    
    // TODO: esse daqui ta apagando o arquivo inteiro, arruma!!

    let data = json!({
        "locate": locate,
        "value": value.to_lowercase(),
        "command": "set"
    });

    JsonSender::send_json_to_server(socket, data);
    let json = JsonSender::receive_json_from_server(socket);
    let mut file = File::create("src/local.json").unwrap();
    file.write_all(json.as_bytes()).unwrap();
}

pub fn get(socket: &mut UdpSocket, locate: String) {
    let data = json!({
        "locate": locate,
        "command": "get"
    });

    JsonSender::send_json_to_server(socket, data);
    let json = JsonSender::receive_json_from_server(socket);
    
    println!("{}", json)
}