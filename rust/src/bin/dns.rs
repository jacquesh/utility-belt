use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use getopts::Options;
use rand::Rng;
use std::{convert::TryInto, env, io, net, process};
use std::time::{Duration, Instant};
use std::str::FromStr;
use std::option::Option;
use std::net::IpAddr;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [OPTIONS] [INPUT]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = &args[0];
    let mut opts_spec = Options::new();
    opts_spec.optflag("h", "help", "print this help menu");
    opts_spec.optflag("r", "reverse", "do a reverse lookup (find the domain name given an IP)");
    opts_spec.optflag("v", "verbose", "print additional data");
    opts_spec.optopt("s", "server", "the address of the server(s) to which the DNS queries should be sent", "IP-ADDR");
    let opts = match opts_spec.parse(&args[1..]) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{}", e.to_string());
            eprintln!("Try '{} --help' for more information", program);
            process::exit(1);
        }
    };

    if opts.opt_present("h") {
        println!("Send a DNS query for the given domain(s)");
        println!("");

        print_usage(program, opts_spec);
        return;
    }

    let verbose = opts.opt_present("v");
    let reverse = opts.opt_present("r");
    let server = match opts.opt_str("s") {
        Some(srv_url) => srv_url,
        None => String::from("1.1.1.1") // TODO: Get the system default
    };

    if opts.free.is_empty() {
        eprintln!("No input provided");
        eprintln!("");
        print_usage(program, opts_spec);
        process::exit(1);
    }

    if opts.free.len() > 1 {
        eprint!("Warning: Cannot lookup multiple domains in a single execution. Ignoring arguments: ");
        for arg in &opts.free[1..] {
            eprint!("{} ", arg);
        }
        eprintln!();
    }

    let query = &opts.free[0];
    let server_ip: IpAddr;
    match IpAddr::from_str(&server) {
        Ok(ip) => server_ip = ip,
        Err(err) => {
            eprintln!("Failed to parse server IP {}: {}", server, err);
            process::exit(1);
        }
    }

    // TODO: Specify record type
    process_input(server_ip, query, QueryType::A, reverse, verbose);
}

#[derive(Clone, Copy, Debug)]
enum DomainClass {
    Reserved = 0,
    Internet = 1,
    Chaos = 3,
    Hesiod = 4
}

impl DomainClass {
    fn from_int(i: u16) -> Option<DomainClass> {
        match i {
            0 => Some(DomainClass::Reserved),
            1 => Some(DomainClass::Internet),
            3 => Some(DomainClass::Chaos),
            4 => Some(DomainClass::Hesiod),
            _ => None
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum QueryType {
    Unknown = 0,
    A = 1,
    NS = 2,
    MD = 3,
    MF = 4,
    CNAME = 5,
    SOA = 6,
    MB = 7,
    MG = 8,
    MR = 9,
    NULL = 10,
    WKS = 11,
    PTR = 12,
    HINFO = 13,
    MINFO = 14,
    MX = 15,
    TXT = 16,
    RP = 17,
    AFSDB = 18,
    X25 = 19,
    ISDN = 20,
    RT = 21,
    NSAP = 22,
    NSAPPTR = 23,
    SIG = 24,
    KEY = 25,
    PX = 26,
    GPOS = 27,
    AAAA = 28,
    LOC = 29,
    NXT = 30,
    EID = 31,
    NIMLOC = 32
}

impl QueryType {
    fn from_int(i: u16) -> Option<QueryType> {
        match i {
            0 => Some(QueryType::Unknown),
            1 => Some(QueryType::A),
            2 => Some(QueryType::NS),
            3 => Some(QueryType::MD),
            4 => Some(QueryType::MF),
            5 => Some(QueryType::CNAME),
            6 => Some(QueryType::SOA),
            7 => Some(QueryType::MB),
            8 => Some(QueryType::MG),
            9 => Some(QueryType::MR),
            10 => Some(QueryType::NULL),
            11 => Some(QueryType::WKS),
            12 => Some(QueryType::PTR),
            13 => Some(QueryType::HINFO),
            14 => Some(QueryType::MINFO),
            15 => Some(QueryType::MX),
            16 => Some(QueryType::TXT),
            17 => Some(QueryType::RP),
            18 => Some(QueryType::AFSDB),
            19 => Some(QueryType::X25),
            20 => Some(QueryType::ISDN),
            21 => Some(QueryType::RT),
            22 => Some(QueryType::NSAP),
            23 => Some(QueryType::NSAPPTR),
            24 => Some(QueryType::SIG),
            25 => Some(QueryType::KEY),
            26 => Some(QueryType::PX),
            27 => Some(QueryType::GPOS),
            28 => Some(QueryType::AAAA),
            29 => Some(QueryType::LOC),
            30 => Some(QueryType::NXT),
            31 => Some(QueryType::EID),
            32 => Some(QueryType::NIMLOC),
            _ => None
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum OpCode {
    Standard = 0,
    Inverse = 1,
    Status = 2
}

impl OpCode {
    fn from_int(i: u16) -> Option<OpCode> {
        match i {
            0 => Some(OpCode::Standard),
            1 => Some(OpCode::Inverse),
            2 => Some(OpCode::Status),
            _ => None
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum ResponseCode {
    NoError = 0,
    FormatError = 1,
    ServerFailure = 2,
    NameError = 3,
    NotImplemented = 4,
    Refused = 5
}

impl ResponseCode {
    fn from_int(i: u16) -> Option<ResponseCode> {
        match i {
            0 => Some(ResponseCode::NoError),
            1 => Some(ResponseCode::FormatError),
            2 => Some(ResponseCode::ServerFailure),
            3 => Some(ResponseCode::NameError),
            4 => Some(ResponseCode::NotImplemented),
            5 => Some(ResponseCode::Refused),
            _ => None
        }
    }
}


#[derive(Debug)]
struct DnsHeader {
    request_id: u16,
    is_response: bool,
    opcode: OpCode,
    authoritative_answer: bool,
    message_truncated: bool,
    recursion_desired: bool,
    recursion_available: bool,
    rcode: ResponseCode,
    query_count: u16,
    answer_count: u16,
    nameserver_count: u16,
    additional_count: u16,
}

impl Default for DnsHeader {
    fn default() -> DnsHeader {
        DnsHeader {
            request_id: 0,
            is_response: false,
            opcode: OpCode::Standard,
            authoritative_answer: false,
            message_truncated: false,
            recursion_desired: false,
            recursion_available: false,
            rcode: ResponseCode::NoError,
            query_count: 0,
            answer_count: 0,
            nameserver_count: 0,
            additional_count: 0,
        }
    }
}

impl DnsHeader {
    fn serialize(&self, buffer: &mut Vec<u8>) -> io::Result<usize> {
        let bitflags: u16 = (u16::from(self.is_response) << 15)
            | ((self.opcode as u16) << 11)
            | (u16::from(self.authoritative_answer) << 10)
            | (u16::from(self.message_truncated) << 9)
            | (u16::from(self.recursion_desired) << 8)
            | (u16::from(self.recursion_available) << 7)
            // 4 0 bits << 4
            | (self.rcode as u16);

        let start_len = buffer.len();
        buffer.write_u16::<BigEndian>(self.request_id)?;
        buffer.write_u16::<BigEndian>(bitflags)?;
        buffer.write_u16::<BigEndian>(self.query_count)?;
        buffer.write_u16::<BigEndian>(self.answer_count)?;
        buffer.write_u16::<BigEndian>(self.nameserver_count)?;
        buffer.write_u16::<BigEndian>(self.additional_count)?;
        let end_len = buffer.len();
        Ok(end_len - start_len)
    }

    fn deserialize(&mut self, cursor: &mut io::Cursor<&[u8]>) -> io::Result<()> {
        self.request_id = cursor.read_u16::<BigEndian>()?;
        let bitflags = cursor.read_u16::<BigEndian>()?;
        match OpCode::from_int((bitflags >> 11) & 0b1111) {
            Some(oc) => {
                self.opcode = oc;
            },
            None => {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid query operation code"));
            }
        }
        self.is_response = ((bitflags >> 15) & 0b1) == 1;
        self.authoritative_answer = ((bitflags >> 10) & 0b1) == 1;
        self.message_truncated = ((bitflags >> 9) & 0b1) == 1;
        self.recursion_desired = ((bitflags >> 8) & 0b1) == 1;
        self.recursion_available = ((bitflags >> 7) & 0b1) == 1;
        match ResponseCode::from_int(bitflags & 0b1111) {
            Some(rc) => {
                self.rcode = rc;
            },
            None => {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid response code"));
            }
        }
        self.query_count = cursor.read_u16::<BigEndian>()?;
        self.answer_count = cursor.read_u16::<BigEndian>()?;
        self.nameserver_count = cursor.read_u16::<BigEndian>()?;
        self.additional_count = cursor.read_u16::<BigEndian>()?;
        Ok(())
    }

    fn deserialize_from(cursor: &mut io::Cursor<&[u8]>) -> io::Result<DnsHeader> {
        let mut result = DnsHeader::default();
        result.deserialize(cursor)?;
        Ok(result)
    }
}

#[derive(Debug)]
struct DnsQuestion {
    domain_name: String,
    query_type: QueryType,
    query_class: DomainClass
}

impl Default for DnsQuestion {
    fn default() -> DnsQuestion {
        DnsQuestion {
            domain_name: String::new(),
            query_type: QueryType::Unknown,
            query_class: DomainClass::Reserved
        }
    }
}

impl DnsQuestion {
    fn serialize(&self, buffer: &mut Vec<u8>) -> io::Result<()> {
        for label in self.domain_name.split('.') {
            buffer.write_u8(label.len().try_into().expect("label length greater than 255"))?;
            for ch in label.bytes() {
                buffer.write_u8(ch)?;
            }
        }
        buffer.write_u8(0)?;
        buffer.write_u16::<BigEndian>(self.query_type as u16)?;
        buffer.write_u16::<BigEndian>(self.query_class as u16)?;
        return Ok(());
    }

    fn deserialize(&mut self, cursor: &mut io::Cursor<&[u8]>) -> io::Result<()> {
        loop {
            let label_len = cursor.read_u8()?;
            if label_len == 0 {
                break;
            }

            if self.domain_name.len() != 0 {
                self.domain_name.push('.');
            }
            for _ in 0..label_len {
                let ch = cursor.read_u8()?;
                self.domain_name.push(char::from(ch));
            }
        }

        match QueryType::from_int(cursor.read_u16::<BigEndian>()?) {
            Some(t) => {
                self.query_type = t;
            },
            None => {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid query type"));
            }
        }
        match DomainClass::from_int(cursor.read_u16::<BigEndian>()?) {
            Some(class) => {
                self.query_class = class;
            },
            None => {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid domain class"));
            }
        }
        Ok(())
    }

    fn deserialize_from(cursor: &mut io::Cursor<&[u8]>) -> io::Result<DnsQuestion> {
        let mut result = DnsQuestion::default();
        result.deserialize(cursor)?;
        Ok(result)
    }
}

#[derive(Debug)]
struct DnsResourceRecord {
    domain_name: String,
    data_type: QueryType,
    data_class: DomainClass,
    ttl: u32,
    data_length: u16,
    data: Vec<u8> // TODO: Maybe we want this to be a &[u8]? To save on copying? Depends on the deserialize
}

impl Default for DnsResourceRecord {
    fn default() -> DnsResourceRecord {
        DnsResourceRecord {
            domain_name: String::new(),
            data_type: QueryType::Unknown,
            data_class: DomainClass::Reserved,
            ttl: 0,
            data_length: 0,
            data: Vec::new()
        }
    }
}

fn deserialize_name(cursor: &mut io::Cursor<&[u8]>, all_data: &[u8]) -> io::Result<Vec<u8>> {
    let mut result = Vec::new();
    loop {
        const POINTER_MASK: u8 = 0b11 << 6;
        let len = cursor.read_u8()?;
        if len == 0 {
            break;
        }

        if result.len() != 0 {
            result.push('.' as u8);
        }

        if len & POINTER_MASK == POINTER_MASK {
            let offset_hi = len & !POINTER_MASK;
            let offset_lo = cursor.read_u8()?;
            let offset = ((offset_hi as u64) << 8) | (offset_lo as u64);
            let mut ptr_cursor = io::Cursor::new(all_data);
            ptr_cursor.set_position(offset);

            let mut pointed_result = deserialize_name(&mut ptr_cursor, all_data)?;
            result.append(&mut pointed_result);
            break;

        } else {
            for _ in 0..len {
                result.push(cursor.read_u8()?);
            }
        }
    }
    Ok(result)
}

impl DnsResourceRecord {
    fn deserialize(&mut self, cursor: &mut io::Cursor<&[u8]>, all_data: &[u8]) -> io::Result<()> {
        let name_bytes = deserialize_name(cursor, all_data)?;
        match String::from_utf8(name_bytes) {
            Ok(name) => { 
                self.domain_name = name;
            },
            Err(e) => {
                return Err(io::Error::new(io::ErrorKind::InvalidData, e));
            }
        }

        match QueryType::from_int(cursor.read_u16::<BigEndian>()?) {
            Some(t) => {
                self.data_type = t;
            },
            None => {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid query type"));
            }
        };
        match DomainClass::from_int(cursor.read_u16::<BigEndian>()?) {
            Some(class) => {
                self.data_class = class;
            },
            None => {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid domain class"));
            }
        };
        self.ttl = cursor.read_u32::<BigEndian>()?;
        self.data_length = cursor.read_u16::<BigEndian>()?;
        self.data.reserve_exact(self.data_length as usize);
        for _ in 0..self.data_length {
            let b = cursor.read_u8()?;
            self.data.push(b);
        }
        Ok(())
    }

    fn deserialize_from(cursor: &mut io::Cursor<&[u8]>, all_data: &[u8]) -> io::Result<DnsResourceRecord> {
        let mut result = DnsResourceRecord::default();
        result.deserialize(cursor, all_data)?;
        Ok(result)
    }
}

#[derive(Debug)]
struct DnsPacket {
    header: DnsHeader,
    questions: Vec<DnsQuestion>,
    answers: Vec<DnsResourceRecord>,
    authorities: Vec<DnsResourceRecord>,
    additionals: Vec<DnsResourceRecord>
}

impl Default for DnsPacket {
    fn default() -> DnsPacket {
        DnsPacket {
            header: DnsHeader::default(),
            questions: Vec::new(),
            answers: Vec::new(),
            authorities: Vec::new(),
            additionals: Vec::new()
        }
    }
}

impl DnsPacket {
    fn serialize(&self, buffer: &mut Vec<u8>) -> io::Result<()> {
        self.header.serialize(buffer)?;

        for question in &self.questions {
            question.serialize(buffer)?;
        }
        return Ok(());
    }

    fn deserialize(&mut self, header: DnsHeader, cursor: &mut io::Cursor<&[u8]>, all_data: &[u8]) -> io::Result<()> {
        self.header = header;
        for _ in 0..self.header.query_count {
            self.questions.push(DnsQuestion::deserialize_from(cursor)?);
        }
        for _ in 0..self.header.answer_count {
            self.answers.push(DnsResourceRecord::deserialize_from(cursor, all_data)?);
        }
        for _ in 0..self.header.nameserver_count {
            self.authorities.push(DnsResourceRecord::deserialize_from(cursor, all_data)?);
        }
        for _ in 0..self.header.additional_count {
            self.additionals.push(DnsResourceRecord::deserialize_from(cursor, all_data)?);
        }
        Ok(())
    }

    fn deserialize_from(header: DnsHeader, cursor: &mut io::Cursor<&[u8]>, all_data: &[u8]) -> io::Result<DnsPacket> {
        let mut result = DnsPacket::default();
        result.deserialize(header, cursor, all_data)?;
        Ok(result)
    }
}

fn process_input(server_ip: IpAddr, domain: &str, qtype: QueryType, reverse: bool, verbose: bool) {
    let mut rng = rand::thread_rng();

    let mut request = DnsPacket::default();
    request.header.request_id = rng.gen::<u16>();
    request.header.recursion_desired = true; // TODO: Maybe we want to be able to ask for no recursion?
    if reverse {
        request.header.opcode = OpCode::Inverse;
    }
    request.header.query_count = 1;

    let mut question = DnsQuestion::default();
    question.domain_name = String::from(domain); // TODO: Can we not just use the &str all the way through?
    question.query_type = qtype;
    question.query_class = DomainClass::Internet;
    request.questions.push(question);

    let mut request_data_buf: Vec<u8> = Vec::new();
    match request.serialize(&mut request_data_buf) {
        Ok(()) => {},
        Err(e) => {
            eprintln!("Failed to serialize request: {}", e);
            process::exit(1);
        }
    }

    const MAX_PORT_SELECT_ATTEMPTS: usize = 15;
    let mut port_select_attempts = 0;
    let socket: net::UdpSocket;
    let socket_addr = IpAddr::from_str("0.0.0.0").unwrap();
    let port_distribution = rand::distributions::Uniform::<u16>::new(49152, 65535); // Taken from the recommendation at https://www.iana.org/assignments/service-names-port-numbers/service-names-port-numbers.xhtml
    loop {
        let port = rng.sample(port_distribution);
        match net::UdpSocket::bind((socket_addr, port)) {
            Ok(new_sock) => {
                socket = new_sock;
                break;
            }
            Err(e) => {
                if port_select_attempts >= MAX_PORT_SELECT_ATTEMPTS {
                    eprintln!("Failed to bind to a local port after {} retries. Terminating.", MAX_PORT_SELECT_ATTEMPTS);
                    process::exit(1);
                } else
                {
                    if verbose {
                        eprintln!("Failed to bind to local port {} ({}), retrying", port, e);
                    }
                    port_select_attempts += 1;
                }
            }
        }
    }

    let dest_addr = net::SocketAddr::from((server_ip, 53));
    socket.set_read_timeout(Some(Duration::from_secs(5))).expect("failed to set socket read timeout");
    socket.set_write_timeout(Some(Duration::from_secs(5))).expect("failed to set socket write timeout");
    let send_instant = Instant::now();
    match socket.send_to(&request_data_buf, dest_addr) {
        Ok(bytes_written) => {
            if verbose {
                println!("Successfully wrote {} bytes of request to the network, waiting for response...", bytes_written);
            }
        },
        Err(e) => {
            eprintln!("Failed to send request to the network: {}", e);
            process::exit(1);
        }
    }

    let mut resp_buffer = [0; 512];
    let (bytes_read, src_addr) = match socket.recv_from(&mut resp_buffer) {
        Ok((bytes_read, src_addr)) => (bytes_read, src_addr),
        Err(e) => {
            eprintln!("Failed to read response from the network: {}", e);
            process::exit(1);
        }
    };
    let roundtrip_ms = (send_instant.elapsed().as_micros() as f64)/1000.0;
    let resp_bytes = &resp_buffer[..bytes_read];

    if src_addr != dest_addr {
        eprintln!("Received answer from unexpected source address: {} Ignoring...", src_addr);
        process::exit(1);
    }

    if verbose {
        print!("Received {} byte response from {} after {:.1}ms - ", bytes_read, src_addr, roundtrip_ms);
        for b in resp_bytes {
            print!("{:x}", b);
        }
        println!();
    } else {
        println!("Received response from {} after {:.1}ms", src_addr, roundtrip_ms);
    }

    let mut cursor = io::Cursor::new(resp_bytes);
    let response_header = match DnsHeader::deserialize_from(&mut cursor) {
        Ok(header) => header,
        Err(e) => {
            eprintln!("Failed to deserialise header from network: {}", e);
            process::exit(1);
        }
    };
    let response = match DnsPacket::deserialize_from(response_header, &mut cursor, resp_bytes) {
        Ok(packet) => packet,
        Err(e) => {
            eprintln!("Failed to deserialise packet from network: {}", e);
            process::exit(1);
        }
    };

    if response.header.request_id != request.header.request_id {
        println!("WARNING: Received packet header contains a request ID that does not match our request!");
    }

    if verbose {
        println!("  Request operation type: {:?}", response.header.opcode);

        if response.header.authoritative_answer {
            println!("  Response is authoritative");
        } else {
            println!("  Response is non-authoritative");
        }

        if response.header.recursion_available {
            println!("  Recursive query resolution is available");
        } else {
            println!("  Recursive query resolution is not available");
        }
    }

    match response.header.rcode {
        ResponseCode::NoError => {
            if verbose {
                println!("  Response code: {:?}", response.header.rcode);
            }
        },
        _ => {
            eprintln!("  Response code: ERROR: {:?}", response.header.rcode);
        }
    }

    if response.header.message_truncated {
        println!("WARNING: Packet header indicates that the data received has been truncated!");
    }

    println!();
    for answer in &response.answers {
        print!("{} ({:?}, {:?}): ", answer.domain_name, answer.data_class, answer.data_type);

        match answer.data_type {
            QueryType::A => {
                const EXPECTED_LEN: usize = 4;
                if answer.data.len() != EXPECTED_LEN {
                    eprintln!("Data type {:?} expects to contain exactly {} bytes and instead contained {}. Ignoring...", answer.data_type, EXPECTED_LEN, answer.data.len());
                } else {
                    println!(" {}.{}.{}.{}", answer.data[0], answer.data[1], answer.data[2], answer.data[3]);
                }
            },
            // TODO: QueryType::AAAA at least?
            _ => {
                eprintln!(" Unsupported response data type: {:?}\n", answer.data_type);
            }
        }
    }

    // TODO: Name server RRs
    // TODO: Additional RRs

    if verbose {
        println!("\n Response packet debug: {:?}", response);
    }
}
