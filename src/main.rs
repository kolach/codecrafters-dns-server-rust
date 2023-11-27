// Uncomment this block to pass the first stage
use std::net::UdpSocket;

#[derive(Debug, Default)]
struct Header {
    // Packet Identifier (ID)
    // 16 bits
    // A random ID assigned to query packets.
    // Response packets must reply with the same ID.
    id: u16,
    // Query/Response Indicator (QR)
    // 1 bit
    // 1 for a reply packet, 0 for a question packet.
    qr: u8,
    // Operation Code (OPCODE)
    // 4 bits
    // Specifies the kind of query in a message.
    opcode: u8,
    // Authoritative Answer (AA)
    // 1 bit
    // 1 if the responding server "owns" the domain queried, i.e., it's authoritative.
    aa: u8,
    // Truncation (TC)
    // 1 bit
    // 1 if the message is larger than 512 bytes.
    // Always 0 in UDP responses.
    tc: u8,
    // Recursion Desired (RD)
    // 1 bit
    // Sender sets this to 1 if the server should recursively resolve this query, 0 otherwise.
    rd: u8,
    // Recursion Available (RA)
    // 1 bit
    // Server sets this to 1 to indicate that recursion is available.
    ra: u8,
    // Reserved (Z)
    // 3 bits
    // Used by DNSSEC queries. At inception, it was reserved for future use.
    z: u8,
    // Response Code (RCODE)
    // 4 bits
    // Response code indicating the status of the response.
    rcode: u8,
    // Question Count (QDCOUNT)
    // 16 bits
    // Number of questions in the Question section.
    qdcount: u16,
    // Answer Record Count (ANCOUNT)
    // 16 bits
    // Number of records in the Answer section.
    ancount: u16,
    // Authority Record Count (NSCOUNT)
    // 16 bits
    // Number of records in the Authority section.
    nscount: u16,
    // Additional Record Count (ARCOUNT)
    // 16 bits
    // Number of records in the Additional section.
    arcount: u16,
}

impl Header {
    fn write_to(&self, buf: &mut Vec<u8>) {
        buf[..2].copy_from_slice(&self.id.to_be_bytes());

        buf[2] |= self.qr << 7;
        buf[2] |= self.opcode << 6;
        buf[2] |= self.aa << 2;
        buf[2] |= self.tc << 1;
        buf[2] |= self.rd;

        buf[3] |= self.ra << 7;
        buf[3] |= self.z << 6;
        buf[3] |= self.rcode << 3;

        buf[4..6].copy_from_slice(&self.qdcount.to_be_bytes());
        buf[6..8].copy_from_slice(&self.ancount.to_be_bytes());
        buf[8..10].copy_from_slice(&self.nscount.to_be_bytes());
        buf[10..12].copy_from_slice(&self.arcount.to_be_bytes());
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(clippy::upper_case_acronyms, dead_code)]
enum Type {
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

impl Type {
    fn write_to(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&(*self as u16).to_be_bytes())
    }
}

impl From<Type> for [u8; 2] {
    fn from(value: Type) -> Self {
        (value as u16).to_be_bytes()
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(clippy::upper_case_acronyms, dead_code)]
enum Class {
    IN = 1, // 1 the Internet
    CS,     // 2 the CSNET class (Obsolete - used only for examples in some obsolete RFCs)
    CH,     // 3 the CHAOS class
    HS,     // 4 Hesiod [Dyer 87]
}

impl Class {
    fn write_to(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&(*self as u16).to_be_bytes())
    }
}

struct Name(String);

impl Name {
    fn write_to(&self, buf: &mut Vec<u8>) {
        for label in self.0.split('.') {
            buf.push(label.len() as u8);
            buf.extend_from_slice(label.as_bytes());
        }
        buf.push(0);
    }
}

struct Question {
    name: Name,
    qtype: Type,
    class: Class,
}

impl Question {
    fn write_to(&self, buf: &mut Vec<u8>) {
        self.name.write_to(buf);
        self.qtype.write_to(buf);
        self.class.write_to(buf);
    }
}

struct Answer {
    name: Name,
    atype: Type,
    class: Class,
    ttl: u32,
    // rdlength: u16,
    rdata: Vec<u8>,
}

impl Answer {
    fn write_to(&self, buf: &mut Vec<u8>) {
        self.name.write_to(buf);
        self.atype.write_to(buf);
        self.class.write_to(buf);

        buf.extend_from_slice(&(self.ttl).to_be_bytes());
        buf.extend_from_slice(&(self.rdata.len() as u16).to_be_bytes());
        buf.extend_from_slice(&self.rdata)
    }
}
struct Message {
    header: Header,
    question: Question,
    answer: Answer,
}

impl Message {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        self.header.write_to(&mut buf);
        self.question.write_to(&mut buf);
        self.answer.write_to(&mut buf);
        buf
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
                        ancount: 1,
                        ..Header::default()
                    },
                    question: Question {
                        name: Name("codecrafters.io".into()),
                        qtype: Type::A,
                        class: Class::IN,
                    },
                    answer: Answer {
                        name: Name("codecrafters.io".into()),
                        atype: Type::A,
                        class: Class::IN,
                        ttl: 60,
                        rdata: vec![8u8; 4],
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
