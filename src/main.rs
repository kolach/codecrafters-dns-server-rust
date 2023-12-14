#[allow(dead_code)]
mod encoder;
#[allow(dead_code)]
mod proto;

use crate::{
    encoder::{Decoder, Encoder},
    proto::Message,
};
use anyhow::Result;
use clap::Parser;
use std::net::{SocketAddr, UdpSocket};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long, value_parser)]
    resolver: SocketAddr,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let fwd_addr = args.resolver;

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

                let mut reply = Message {
                    id: request.id,
                    opcode: request.opcode,
                    rd: request.rd,
                    rcode: if request.opcode == 0 { 0 } else { 4 },
                    qr: 1,
                    questions: request.questions.clone(),
                    ..Message::default()
                };

                println!("---> Parsed request: {:?}", request);

                let fwd_socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bin fwd socket");

                for question in request.questions.iter() {
                    let mut fwd_request = request.clone();
                    fwd_request.questions = vec![question.clone()];
                    let mut buf = Vec::with_capacity(512);
                    let mut enc = Encoder::new(&mut buf);
                    fwd_request.encode(&mut enc)?;

                    println!("---> Sending query to fwd server: {:?}", fwd_request);

                    fwd_socket
                        .send_to(&buf, fwd_addr.to_string())
                        .expect("failed to send forward request");

                    let mut response_buf = [0u8; 512];
                    let (_, _) = fwd_socket.recv_from(&mut response_buf)?;
                    let mut dec = Decoder::new(&mut response_buf);
                    let fwd_reply = Message::decode(&mut dec)?;

                    println!("<--- Parsed reply from fwd server: {:?}", fwd_reply);

                    for answer in fwd_reply.answers.into_iter() {
                        reply.answers.push(answer);
                    }
                }

                // let answers = request
                //     .questions
                //     .iter()
                //     .map(|q| Record {
                //         name: q.name.clone(),
                //         rtype: q.qtype,
                //         class: q.class,
                //         ttl: 60,
                //         rdata: vec![8u8; 4],
                //     })
                //     .collect();
                //
                // let reply = Message {
                //     id: request.id,
                //     opcode: request.opcode,
                //     rd: request.rd,
                //     rcode: if request.opcode == 0 { 0 } else { 4 },
                //     qr: 1,
                //     questions: request.questions,
                //     answers,
                //     ..Message::default()
                // };

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
