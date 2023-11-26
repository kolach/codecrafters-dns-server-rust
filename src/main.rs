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

impl From<Header> for [u8; 12] {
    // 12 bytes header:
    //
    // id takes first 2 bytes
    //
    // qr 1 bit
    // opcode 4 bits
    // aa 1 bit
    // tc 1 bit
    // rd 1 bit
    //
    // ra 1 bit
    // z 4 bits
    // rcode 3 bits
    //
    // qdcount 2 bytes
    // ancount 2 bytes
    // nscount 2 bytes
    // arcount 2 count
    fn from(value: Header) -> Self {
        let mut buf = [0u8; 12];

        buf[..2].copy_from_slice(&value.id.to_be_bytes());

        buf[2] |= value.qr << 7;
        buf[2] |= value.opcode << 6;
        buf[2] |= value.aa << 2;
        buf[2] |= value.tc << 1;
        buf[2] |= value.rd;

        buf[3] |= value.ra << 7;
        buf[3] |= value.z << 6;
        buf[3] |= value.rcode << 3;

        buf[4..6].copy_from_slice(&value.qdcount.to_be_bytes());
        buf[6..8].copy_from_slice(&value.ancount.to_be_bytes());
        buf[8..10].copy_from_slice(&value.nscount.to_be_bytes());
        buf[10..12].copy_from_slice(&value.arcount.to_be_bytes());

        buf
    }
}

#[allow(clippy::upper_case_acronyms, dead_code)]
enum QuestionType {
    A = 1, // 1 a host address
    NS,    // 2 an authoritative name server
    MD,    // 3 a mail destination (Obsolete - use MX)
    MF,    // 4 a mail forwarder (Obsolete - use MX)
    CNAME, // 5 the canonical name for an alias
    SOA,   // 6 marks the start of a zone of authority
    MB,    // 7 a mailbox domain name (EXPERIMENTAL)
    MG,    // 8 a mail group member (EXPERIMENTAL)
    MR,    // 9 a mail rename domain name (EXPERIMENTAL)
    NULL,  // 10 a null RR (EXPERIMENTAL)
    WKS,   // 11 a well known service description
    PTR,   // 12 a domain name pointer
    HINFO, // 13 host information
    MINFO, // 14 mailbox or mail list information
    MX,    // 15 mail exchange
    TXT,   // 16 text strings
}

#[allow(clippy::upper_case_acronyms, dead_code)]
enum QuestionClass {
    IN = 1, // 1 the Internet
    CS,     // 2 the CSNET class (Obsolete - used only for examples in some obsolete RFCs)
    CH,     // 3 the CHAOS class
    HS,     // 4 Hesiod [Dyer 87]
}

struct Question {
    name: String,
    qtype: QuestionType,
    class: QuestionClass,
}

impl From<Question> for Vec<u8> {
    fn from(value: Question) -> Self {
        let mut buf = Vec::new();
        for label in value.name.split('.') {
            let len = label.len() as u8;
            buf.push(len);
            buf.extend_from_slice(label.as_bytes());
        }
        buf.push(0);
        buf.extend_from_slice(&(value.qtype as u16).to_be_bytes());
        buf.extend_from_slice(&(value.class as u16).to_be_bytes());
        buf
    }
}

struct Message {
    header: Header,
    question: Question,
}

impl From<Message> for Vec<u8> {
    fn from(value: Message) -> Self {
        let mut bytes = Vec::new();

        let header_bytes: [u8; 12] = value.header.into();
        bytes.extend_from_slice(&header_bytes);

        let question_bytes: Vec<u8> = value.question.into();
        bytes.extend_from_slice(&question_bytes);

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

                let response: Vec<u8> = Message {
                    header: Header {
                        id: 1234,
                        qr: 1,
                        qdcount: 1,
                        ..Header::default()
                    },
                    question: Question {
                        name: "codecrafters.io".into(),
                        qtype: QuestionType::A,
                        class: QuestionClass::IN,
                    },
                }
                .into();

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
