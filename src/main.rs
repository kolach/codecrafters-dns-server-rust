// Uncomment this block to pass the first stage
use std::net::UdpSocket;

#[derive(Default)]
struct Header {
    id: u16,
    qr: u8,
    opcode: u8,
    aa: u8,
    tc: u8,
    rd: u8,
    ra: u8,
    z: u8,
    rcode: u8,
    qdcount: u16,
    ancount: u16,
    nscount: u16,
    arcount: u16,
}

impl Header {
    // id takes first 2 bytes
    // qr 1 bit
    // opcode 4 bits
    // aa 1 bit
    // tc 1 bit
    // rd 1 bit
    fn to_bytes(&self) -> [u8; 12] {
        let mut buf = [0u8; 12];

        buf[..2].copy_from_slice(&self.id.to_be_bytes());

        buf[2] = (self.qr << 7) | (self.opcode << 6) | (self.aa << 2) | (self.tc << 1) | (self.rd);

        buf[3] |= self.ra << 7;
        buf[3] |= self.z << 6;
        buf[3] |= self.rcode << 3;

        buf[4..6].copy_from_slice(&self.qdcount.to_be_bytes());
        buf[6..8].copy_from_slice(&self.ancount.to_be_bytes());
        buf[8..10].copy_from_slice(&self.nscount.to_be_bytes());
        buf[10..12].copy_from_slice(&self.arcount.to_be_bytes());

        buf
    }
}

struct Message {
    header: Header,
}

impl Message {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        let header_bytes = self.header.to_bytes();
        bytes.extend_from_slice(&header_bytes);
        bytes
    }
}

fn main() {
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

                let response = Message {
                    header: Header {
                        id: 1234,
                        qr: 1,
                        ..Header::default()
                    },
                }
                .to_bytes();

                udp_socket
                    .send_to(&response, source)
                    .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
}
