use std::net::UdpSocket;
use std::time::Duration;

use std::env;
use wireguard_proxy::Args;

const PONG: [u8; 246] = [
    0x6A, 0x2, 0x6B, 0xC, 0x6C, 0x3F, 0x6D, 0xC, 0xA2, 0xEA, 0xDA, 0xB6, 0xDC, 0xD6, 0x6E, 0x0,
    0x22, 0xD4, 0x66, 0x3, 0x68, 0x2, 0x60, 0x60, 0xF0, 0x15, 0xF0, 0x7, 0x30, 0x0, 0x12, 0x1A,
    0xC7, 0x17, 0x77, 0x8, 0x69, 0xFF, 0xA2, 0xF0, 0xD6, 0x71, 0xA2, 0xEA, 0xDA, 0xB6, 0xDC, 0xD6,
    0x60, 0x1, 0xE0, 0xA1, 0x7B, 0xFE, 0x60, 0x4, 0xE0, 0xA1, 0x7B, 0x2, 0x60, 0x1F, 0x8B, 0x2,
    0xDA, 0xB6, 0x60, 0xC, 0xE0, 0xA1, 0x7D, 0xFE, 0x60, 0xD, 0xE0, 0xA1, 0x7D, 0x2, 0x60, 0x1F,
    0x8D, 0x2, 0xDC, 0xD6, 0xA2, 0xF0, 0xD6, 0x71, 0x86, 0x84, 0x87, 0x94, 0x60, 0x3F, 0x86, 0x2,
    0x61, 0x1F, 0x87, 0x12, 0x46, 0x2, 0x12, 0x78, 0x46, 0x3F, 0x12, 0x82, 0x47, 0x1F, 0x69, 0xFF,
    0x47, 0x0, 0x69, 0x1, 0xD6, 0x71, 0x12, 0x2A, 0x68, 0x2, 0x63, 0x1, 0x80, 0x70, 0x80, 0xB5,
    0x12, 0x8A, 0x68, 0xFE, 0x63, 0xA, 0x80, 0x70, 0x80, 0xD5, 0x3F, 0x1, 0x12, 0xA2, 0x61, 0x2,
    0x80, 0x15, 0x3F, 0x1, 0x12, 0xBA, 0x80, 0x15, 0x3F, 0x1, 0x12, 0xC8, 0x80, 0x15, 0x3F, 0x1,
    0x12, 0xC2, 0x60, 0x20, 0xF0, 0x18, 0x22, 0xD4, 0x8E, 0x34, 0x22, 0xD4, 0x66, 0x3E, 0x33, 0x1,
    0x66, 0x3, 0x68, 0xFE, 0x33, 0x1, 0x68, 0x2, 0x12, 0x16, 0x79, 0xFF, 0x49, 0xFE, 0x69, 0xFF,
    0x12, 0xC8, 0x79, 0x1, 0x49, 0x2, 0x69, 0x1, 0x60, 0x4, 0xF0, 0x18, 0x76, 0x1, 0x46, 0x40,
    0x76, 0xFE, 0x12, 0x6C, 0xA2, 0xF2, 0xFE, 0x33, 0xF2, 0x65, 0xF1, 0x29, 0x64, 0x14, 0x65, 0x0,
    0xD4, 0x55, 0x74, 0x15, 0xF2, 0x29, 0xD4, 0x55, 0x0, 0xEE, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80,
    0x80, 0x0, 0x0, 0x0, 0x0, 0x0,
];

struct Server {
    udp_host: String,
    udp_target: String,
    socket_timeout: Option<Duration>,
}

impl Server {
    fn new(udp_host: String, udp_target: String, secs: u64) -> Server {
        Server {
            udp_host,
            udp_target,
            socket_timeout: match secs {
                0 => None,
                x => Some(Duration::from_secs(x)),
            },
        }
    }

    fn start(&self) -> std::io::Result<usize> {
        let udp_host = UdpSocket::bind(&self.udp_host)?;
        udp_host.set_read_timeout(self.socket_timeout)?;

        let udp_target = UdpSocket::bind("127.0.0.1:3401")?;
        udp_target.connect(&self.udp_target)?;

        udp_target.send(&PONG)?;

        let mut buf = [0u8; 2048];
        match udp_host.recv(&mut buf) {
            Ok(len) => {
                println!("udp got len: {}", len);
                assert_eq!(len, PONG.len());
                assert_eq!(&buf[..len], &PONG[..]);
            }
            Err(e) => {
                panic!("recv function failed: {:?}", e);
            }
        }

        println!("success! received back exactly what we sent!");

        Ok(0)
    }
}

fn main() {
    let raw_args = env::args().collect();
    let args = Args::new(&raw_args);
    if args.get_str(1, "").contains("-h") {
        println!(
            "usage: {} [-h] [udp_host, 127.0.0.1:51821] [udp_target, 127.0.0.1:51821] [socket_timeout, 1]",
            args.get_str(0, "udp-test")
        );
        return;
    }

    let server = Server::new(
        args.get_str(1, "127.0.0.1:51821").to_owned(),
        args.get_str(2, "127.0.0.1:51821").to_owned(),
        args.get(3, 1),
    );

    println!(
        "udp_host: {}, udp_target: {}, socket_timeout: {:?}",
        server.udp_host, server.udp_target, server.socket_timeout,
    );

    server.start().expect("error running server");
}
