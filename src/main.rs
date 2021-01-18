mod handle;
use clap::{App, Arg};
use std::net::*;
use threadpool::ThreadPool;

fn main() {
    let matches = App::new("fileserv-simple")
        .version("0.1.0")
        .arg(
            Arg::with_name("workers")
                .short("w")
                .long("workers")
                .value_name("AMOUNT")
                .help("Number of requests that can be handled concurrently [default: 10]")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .value_name("PORT")
                .help("Port to run on [default: 8080]")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("bind")
                .short("b")
                .long("bind")
                .value_name("ADDRESS")
                .help("Alternative bind address [default: 127.0.0.1]")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("directory")
                .short("d")
                .long("directory")
                .value_name("DIRECTORY")
                .help("Alternative directory to serve [deafault: curent directory")
                .takes_value(true),
        )
        .get_matches();

    let n_workers: usize = matches
        .value_of("workers")
        .unwrap_or("10")
        .parse()
        .expect("Args Error: Invalid worker count");
    let port: u16 = matches
        .value_of("port")
        .unwrap_or("8080")
        .parse()
        .expect("Args Error: Invalid port number");
    let bind = matches.value_of("bind").unwrap_or("127.0.0.1");
    let directory = matches.value_of("directory").unwrap_or(".");

    let listener = TcpListener::bind((bind, port)).expect("Error: Failed to bind to port");
    let pool = ThreadPool::new(n_workers);

    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            let dir = String::from(directory);
            pool.execute(move || {
                let _res = handle::handle_connection(&stream, dir);
            });
        }
    }
}
