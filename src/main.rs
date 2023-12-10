#[allow(dead_code)]
mod encoder;
#[allow(dead_code)]
mod proto;

// Uncomment this block to pass the first stage
use std::net::UdpSocket;

use crate::{
    encoder::{Decoder, Encoder, Error},
    proto::{Message, Record},
};

fn main() -> Result<(), Error> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                let _received_data = String::from_utf8_lossy(&buf[0..size]);
                println!("Received {} bytes from {}", size, source);

                let mut dec = Decoder::new(&buf);
                let request = Message::decode(&mut dec)?;

                println!("---> Parsed request: {:?}", request);

                let answers = request
                    .questions
                    .iter()
                    .map(|q| Record {
                        name: q.name.clone(),
                        rtype: q.qtype,
                        class: q.class,
                        ttl: 60,
                        rdata: vec![8u8; 4],
                    })
                    .collect();

                let reply = Message {
                    id: request.id,
                    opcode: request.opcode,
                    rd: request.rd,
                    rcode: if request.opcode == 0 { 0 } else { 4 },
                    qr: 1,
                    questions: request.questions,
                    answers,
                    ..Message::default()
                };

                let mut buf = Vec::new();
                let mut enc = Encoder::new(&mut buf);
                reply.encode(&mut enc)?;

                udp_socket
                    .send_to(&buf, source)
                    .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
    Ok(())
}
