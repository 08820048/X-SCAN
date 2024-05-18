use ansi_term::Color::Red;
use bpaf::Bpaf;
use prettytable::{Cell, Row, Table};
use std::io::{self, Write};
use std::net::{IpAddr, Ipv4Addr};
use std::sync::mpsc::{channel, Sender};
use tokio::net::TcpStream;
use tokio::task;

// 最大端口号
const MAX: u16 = 65535;

const IPFALLBACK: IpAddr = IpAddr::V4(Ipv4Addr::new(12, 0, 0, 1));

// 命令行参数定义
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Argument {
    #[bpaf(long, short, argument("Address"), fallback(IPFALLBACK))]
    /// The address that you want to sniff.  Must be a valid ipv4 address.  Falls back to 127.0.0.1
    pub address: IpAddr,
    #[bpaf(
        long("start"),
        short('s'),
        guard(start_port_guard, "Must be greater than 0"),
        fallback(1u16)
    )]
    pub start_port: u16,

    #[bpaf(
        long("end"),
        short('e'),
        guard(end_port_guard, "Must be less than or equal to 65535"),
        fallback(MAX)
    )]
    pub end_port: u16,
}

fn start_port_guard(input: &u16) -> bool {
    *input > 0
}

fn end_port_guard(input: &u16) -> bool {
    *input <= MAX
}

// 异步扫描

async fn scan(tx: Sender<u16>, start_port: u16, addr: IpAddr) {
    match TcpStream::connect(format!("{}:{}", addr, start_port)).await {
        Ok(_) => {
            print!(">>>");
            io::stdout().flush().unwrap();
            tx.send(start_port).unwrap();
        }
        Err(_) => {}
    }
}

fn print_infos() {
    println!(
        "{}",
        Red.paint(
            r#"
         __   __            _____    _____              _   _ 
         \ \ / /           / ____|  / ____|     /\     | \ | |
          \ V /   ______  | (___   | |         /  \    |  \| |
           > <   |______|  \___ \  | |        / /\ \   | . ` |
          / . \            ____) | | |____   / ____ \  | |\  |
         /_/ \_\          |_____/   \_____| /_/    \_\ |_| \_|
                                                              
        author:代号0408
        version:0.1.0                                                      
        "#
        )
    );
}

#[tokio::main]
async fn main() {
    print_infos();
    let opts = argument().run();

    let (tx, rx) = channel();

    for i in opts.start_port..opts.end_port {
        let tx = tx.clone();
        task::spawn(async move { scan(tx, i, opts.address).await });
    }
    let mut open_ports = vec![];

    drop(tx);

    for p in rx {
        open_ports.push(p);
    }

    println!("");
    open_ports.sort();

    let mut table = Table::new();
    table.add_row(Row::new(vec![
        Cell::new("Port").style_spec("Fg=blue"),
        Cell::new("Status").style_spec("Fg=blue"),
    ]));

    for port in open_ports {
        table.add_row(Row::new(vec![
            Cell::new(&port.to_string()),
            Cell::new("is open"),
        ]));
    }

    table.printstd();
}
