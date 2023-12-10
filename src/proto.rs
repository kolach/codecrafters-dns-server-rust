use crate::encoder::{Decoder, Encoder, Error};

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Name(pub String);

impl Name {
    pub fn encode(&self, enc: &mut Encoder) {
        for label in self.0.split('.') {
            enc.write_u8(label.len() as u8);
            enc.write_str(label);
        }
        enc.write_u8(0);
    }

    pub fn decode(dec: &mut Decoder) -> Result<Self, Error> {
        let mut segments = Vec::new();
        let mut len = dec.read_u8()? as usize;

        while len > 0 {
            let bytes = dec.read_slice(len)?;
            let label = std::str::from_utf8(bytes)?;
            segments.push(label);
            len = dec.read_u8()? as usize;
        }

        Ok(Self(segments.join(".")))
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[allow(clippy::upper_case_acronyms, dead_code)]
pub enum Type {
    #[default]
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
    pub fn encode(&self, enc: &mut Encoder) {
        enc.write_u16(*self as u16)
    }

    pub fn decode(dec: &mut Decoder) -> Result<Self, Error> {
        let value = dec.read_u16()?;
        match value {
            1 => Ok(Self::A),
            2 => Ok(Self::NS),
            3 => Ok(Self::MD),
            4 => Ok(Self::MF),
            5 => Ok(Self::CNAME),
            6 => Ok(Self::SOA),
            7 => Ok(Self::MB),
            8 => Ok(Self::MG),
            9 => Ok(Self::MR),
            10 => Ok(Self::NULL),
            11 => Ok(Self::WKS),
            12 => Ok(Self::PTR),
            13 => Ok(Self::HINFO),
            14 => Ok(Self::MINFO),
            15 => Ok(Self::MX),
            16 => Ok(Self::TXT),
            _ => Err(Error::Custom(format!("wrong class code {}", value))),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[allow(clippy::upper_case_acronyms, dead_code)]
pub enum Class {
    #[default]
    IN = 1, // 1 the Internet
    CS, // 2 the CSNET class (Obsolete - used only for examples in some obsolete RFCs)
    CH, // 3 the CHAOS class
    HS, // 4 Hesiod [Dyer 87]
}

impl Class {
    pub fn encode(&self, enc: &mut Encoder) {
        enc.write_u16(*self as u16)
    }

    pub fn decode(dec: &mut Decoder) -> Result<Self, Error> {
        let value = dec.read_u16()?;
        match value {
            1 => Ok(Self::IN),
            2 => Ok(Self::CS),
            3 => Ok(Self::CH),
            4 => Ok(Self::HS),
            _ => Err(Error::Custom(format!("wrong class code {}", value))),
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Question {
    pub name: Name,
    pub qtype: Type,
    pub class: Class,
}

impl Question {
    fn encode(&self, enc: &mut Encoder) {
        self.name.encode(enc);
        enc.write_u16(self.qtype as u16);
        enc.write_u16(self.class as u16);
    }

    fn decode(dec: &mut Decoder) -> Result<Self, Error> {
        let mut q = Self::default();
        q.name = Name::decode(dec)?;
        q.qtype = Type::decode(dec)?;
        q.class = Class::decode(dec)?;
        Ok(q)
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct Record {
    pub name: Name,
    pub rtype: Type,
    pub class: Class,
    pub ttl: u32,
    // rdlength: u16, taken from rdata
    pub rdata: Vec<u8>,
}

impl Record {
    pub fn encode(&self, enc: &mut Encoder) {
        self.name.encode(enc);
        enc.write_u16(self.rtype as u16);
        enc.write_u16(self.class as u16);
        enc.write_u32(self.ttl);
        enc.write_u16(self.rdata.len() as u16);
        enc.write_slice(&self.rdata)
    }

    pub fn decode(dec: &mut Decoder) -> Result<Self, Error> {
        let mut rec = Record::default();

        rec.name = Name::decode(dec)?;
        rec.rtype = Type::decode(dec)?;
        rec.class = Class::decode(dec)?;
        rec.ttl = dec.read_u32()?;
        let rdlength = dec.read_u16()?;
        rec.rdata = dec.read_slice(rdlength as usize)?.to_vec();
        Ok(rec)
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct Message {
    // Packet Identifier (ID), 16 bits
    // A random ID assigned to query packets.
    // Response packets must reply with the same ID.
    pub id: u16,

    // Query/Response Indicator (QR), 1 bit
    // 1 for a reply packet, 0 for a question packet.
    pub qr: u8,

    // Operation Code (OPCODE), 4 bits
    // Specifies the kind of query in a message.
    pub opcode: u8,

    // Authoritative Answer (AA), 1 bit
    // 1 if the responding server "owns" the domain queried, i.e., it's authoritative.
    pub aa: u8,

    // Truncation (TC), 1 bit
    // 1 if the message is larger than 512 bytes.
    // Always 0 in UDP responses.
    pub tc: u8,

    // Recursion Desired (RD), 1 bit
    // Sender sets this to 1 if the server should recursively resolve this query, 0 otherwise.
    pub rd: u8,

    // Recursion Available (RA), 1 bit
    // Server sets this to 1 to indicate that recursion is available.
    pub ra: u8,

    // Reserved (Z), 3 bits
    // Used by DNSSEC queries. At inception, it was reserved for future use.
    pub z: u8,
    // Response Code (RCODE), 4 bits
    // Response code indicating the status of the response.
    pub rcode: u8,

    // Authority Record Count (NSCOUNT), 16 bits
    // Number of records in the Authority section.
    pub nscount: u16,
    // Additional Record Count (ARCOUNT), 16 bits
    // Number of records in the Additional section.
    pub arcount: u16,

    // questions
    pub questions: Vec<Question>,

    // answers
    pub answers: Vec<Record>,
}

impl Message {
    pub fn encode(&self, enc: &mut Encoder) -> Result<(), Error> {
        enc.write_u16(self.id);
        enc.write_bits(|b| {
            b.write(self.qr, 1)?;
            b.write(self.opcode, 4)?;
            b.write(self.aa, 1)?;
            b.write(self.tc, 1)?;
            b.write(self.rd, 1)
        })?;
        enc.write_bits(|b| {
            b.write(self.ra, 1)?;
            b.write(self.z, 3)?;
            b.write(self.rcode, 4)
        })?;
        enc.write_u16(self.questions.len() as u16);
        enc.write_u16(self.answers.len() as u16);
        enc.write_u16(self.nscount);
        enc.write_u16(self.arcount);

        self.questions.iter().for_each(|q| q.encode(enc));
        self.answers.iter().for_each(|a| a.encode(enc));
        Ok(())
    }

    pub fn decode(dec: &mut Decoder) -> Result<Self, Error> {
        let mut msg = Message::default();

        msg.id = dec.read_u16()?;
        dec.read_bits(|b| {
            msg.qr = b.read(1)?;
            msg.opcode = b.read(4)?;
            msg.aa = b.read(1)?;
            msg.tc = b.read(1)?;
            msg.rd = b.read(1)?;
            Ok(())
        })?;
        dec.read_bits(|b| {
            msg.ra = b.read(1)?;
            msg.z = b.read(3)?;
            msg.rcode = b.read(4)?;
            Ok(())
        })?;

        let qdcount = dec.read_u16()?;
        let ancount = dec.read_u16()?;
        msg.nscount = dec.read_u16()?;
        msg.arcount = dec.read_u16()?;

        // now we read questions based on qdcount from header
        msg.questions = (0..qdcount)
            .into_iter()
            .map(|_| Question::decode(dec))
            .collect::<Result<Vec<_>, _>>()?;

        msg.answers = (0..ancount)
            .into_iter()
            .map(|_| Record::decode(dec))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(msg)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut buf = Vec::with_capacity(512);
        let mut enc = Encoder::new(&mut buf);
        self.encode(&mut enc)?;
        Ok(buf)
    }

    pub fn from_bytes(buf: &[u8]) -> Result<Self, Error> {
        let mut dec = Decoder::new(buf);
        let msg = Self::decode(&mut dec)?;
        Ok(msg)
    }
}

#[cfg(test)]
mod test {
    use super::{Class, Decoder, Encoder, Message, Name, Question, Record, Type};

    fn test_cases() -> Vec<(&'static str, Vec<u8>)> {
        vec![
            (
                "codecrafters.io",
                vec![
                    12, b'c', b'o', b'd', b'e', b'c', b'r', b'a', b'f', b't', b'e', b'r', b's', 2,
                    b'i', b'o', 0,
                ],
            ),
            (
                "api.github.com",
                vec![
                    3, b'a', b'p', b'i', 6, b'g', b'i', b't', b'h', b'u', b'b', 3, b'c', b'o',
                    b'm', 0,
                ],
            ),
        ]
    }

    #[test]
    fn test_name_encode() {
        for (input, expect) in test_cases() {
            let name = Name(input.into());
            let mut buf = Vec::new();
            let mut encoder = Encoder::new(&mut buf);
            name.encode(&mut encoder);
            assert_eq!(expect, buf);
        }
    }

    #[test]
    fn test_name_decode() {
        for (expect, input) in test_cases() {
            let mut decoder = Decoder::new(&input);
            let name = Name::decode(&mut decoder);
            assert_eq!(Ok(Name(expect.into())), name);
        }
    }

    #[test]
    fn test_msg_encode_decode() {
        let orig_msg = Message {
            id: 1,
            aa: 1,
            questions: vec![Question {
                name: Name("codecrafters.io".into()),
                qtype: Type::A,
                class: Class::IN,
            }],
            answers: vec![Record {
                name: Name("codecrafters.io".into()),
                rtype: Type::A,
                class: Class::IN,
                ttl: 60,
                rdata: vec![8u8; 4],
            }],
            ..Message::default()
        };

        let mut buf = Vec::new();
        let mut enc = Encoder::new(&mut buf);
        let res = orig_msg.encode(&mut enc);

        assert!(res.is_ok());

        let mut dec = Decoder::new(&mut buf);
        let res = Message::decode(&mut dec);
        assert_eq!(Ok(orig_msg), res);
    }
}
