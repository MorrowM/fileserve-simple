mod handle;
use clap::{App, Arg};
use std::net::*;
use threadpool::ThreadPool;

const DEFAULT_WORKERS: &str = "10";
const DEFAULT_PORT: &str = "8080";
const DEFAULT_BIND: &str = "0.0.0.0";

fn main() {
    let matches = App::new("fileserve-simple")
        .version("0.1.0")
        .arg(
            Arg::with_name("workers")
                .short("w")
                .long("workers")
                .value_name("AMOUNT")
                .help(&format!(
                    "Number of requests that can be handled concurrently [default: {}]",
                    DEFAULT_WORKERS
                ))
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .value_name("PORT")
                .help(&format!("Port to run on [default: {}]", DEFAULT_PORT))
                .takes_value(true),
        )
        .arg(
            Arg::with_name("bind")
                .short("b")
                .long("bind")
                .value_name("ADDRESS")
                .help(&format!(
                    "Alternative bind address [default: {}]",
                    DEFAULT_BIND
                ))
                .takes_value(true),
        )
        .arg(
            Arg::with_name("directory")
                .short("d")
                .long("directory")
                .value_name("DIRECTORY")
                .help("Alternative directory to serve [default: curent directory]")
                .takes_value(true),
        )
        .get_matches();

    let n_workers: usize = matches
        .value_of("workers")
        .unwrap_or(DEFAULT_WORKERS)
        .parse()
        .expect("Args Error: Invalid worker count");
    let port: u16 = matches
        .value_of(DEFAULT_PORT)
        .unwrap_or("8080")
        .parse()
        .expect("Args Error: Invalid port number");
    let bind = matches.value_of("bind").unwrap_or(DEFAULT_BIND);
    let directory = matches.value_of("directory").unwrap_or(".");

    let listener = TcpListener::bind((bind, port)).expect("Error: Failed to bind to port");
    let pool = ThreadPool::new(n_workers);

    println!(
        "Serving HTTP on {} port {} (http://{}:{}/) ...",
        bind, port, bind, port
    );

    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let dir = String::from(directory);
            pool.execute(move || {
                let _res = handle::handle_connection(&mut stream, dir);
            });
        }
    }
}
