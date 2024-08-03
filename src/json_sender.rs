use std::net::{SocketAddr, UdpSocket};
use serde_json::Value;

pub struct JsonSender {

}

impl JsonSender {
    pub fn send_json_to_server(connection: &UdpSocket, data: Value, server_addr: SocketAddr) {
        let json_string = serde_json::to_string(&data).expect("Falha ao serializar JSON");
        let metadata = json_string.len() as u64;
        let bytes = json_string.as_bytes();
        
        connection.connect(server_addr).expect("");

        connection.send(&metadata.to_le_bytes()).expect("TODO: panic message");
        connection.send(bytes).unwrap();
    }
    
    pub fn receive_json_from_server(connection: &UdpSocket) -> String {
        let mut buffer = [0; 8];
        connection.recv_from(&mut buffer).expect("");
        let json_size = u64::from_le_bytes(buffer);

        let mut json_buffer = vec![0; json_size as usize];
        connection.recv_from(&mut json_buffer).expect("");
        let json = String::from_utf8_lossy(&json_buffer[..]);

        return json.to_string();
    }
}

