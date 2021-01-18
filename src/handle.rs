use httparse::Request;
use std::fs::DirEntry;
use std::fs::File;
use std::io::prelude::*;
use std::net::*;
use std::path::Path;

pub enum HttpError {
    FailedRead(std::io::Error),
    FailedParse(httparse::Error),
    MissingField(HttpField),
    FailedWrite(std::io::Error),
}

pub enum HttpField {
    Version,
    Method,
    Path,
}

pub fn handle_connection(mut stream: &mut TcpStream, directory: String) -> Result<(), HttpError> {
    let mut buf = [0; 1024];
    stream.read(&mut buf).map_err(HttpError::FailedRead)?;

    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = Request::new(&mut headers);
    let _status = req.parse(&buf).map_err(HttpError::FailedParse)?;

    handle_request(&req, &mut stream, &directory)?;

    Ok(())
}

fn handle_request(
    req: &Request,
    mut stream: &mut TcpStream,
    directory: &str,
) -> Result<(), HttpError> {
    let (version, method, path) = all_fields(&req)?;
    println!(
        "Request:\n{} {} {} from {:?}",
        version,
        method,
        path,
        stream.peer_addr()
    );

    match fetch_path(path, directory) {
        Ok(FetchResult::Dir(html)) => {
            stream
                .write(
                    format!(
                        "HTTP/1.1 200 Ok\n\
                Content-Type: text/html; charset=utf-8\n\n{}",
                        html
                    )
                    .as_bytes(),
                )
                .map_err(HttpError::FailedWrite)?;
        }
        Ok(FetchResult::File(mut file)) => {
            send_file(&mut stream, &mut file).map_err(HttpError::FailedWrite)?;
        }
        Err(FetchError::FileNotFound) => {
            stream
                .write(
                    String::from(
                        "HTTP/1.1 404 Not Found\n\
        Content-Type: text/html; charset=utf-8\n\n\
        <h1> Error: File Not Found",
                    )
                    .as_bytes(),
                )
                .map_err(HttpError::FailedWrite)?;
        }
        Err(FetchError::IOError(_)) => {
            stream
                .write(
                    String::from(
                        "HTTP/1.1 500 Server Error\n\
            Content-Type: text/html; charset=utf-8\n\n\
            <h1> 500 Intenal Error",
                    )
                    .as_bytes(),
                )
                .map_err(HttpError::FailedWrite)?;
        }
    };
    Ok(())
}

fn send_file(stream: &mut TcpStream, file: &mut File) -> Result<(), std::io::Error> {
    let _sent = stream.write(String::from("HTTP/1.1 200 Ok\n\n\n").as_bytes())?;

    let mut buf: [u8; 8192] = [0; 8192];
    loop {
        let amount = file.read(&mut buf)?;
        if amount > 0 {
            let _sent = stream.write(&buf[0..amount])?;
        } else {
            break Ok(());
        }
    }
}

fn all_fields<'r>(req: &'r Request) -> Result<(u8, &'r str, &'r str), HttpError> {
    let version = req
        .version
        .ok_or(HttpError::MissingField(HttpField::Version))?;
    let method = req
        .method
        .ok_or(HttpError::MissingField(HttpField::Method))?;
    let path = req.path.ok_or(HttpError::MissingField(HttpField::Path))?;
    Ok((version, method, path))
}

enum FetchError {
    FileNotFound,
    IOError(std::io::Error),
}

enum FetchResult {
    Dir(String),
    File(File),
}

fn fetch_path(path_str: &str, directory: &str) -> Result<FetchResult, FetchError> {
    let mut path_string = String::from(directory);
    path_string.push_str(path_str);
    let path = Path::new(&path_string);
    if path.is_dir() {
        let title = format!("Directory listing for {}", path_str);
        let start = format!(
            "<!DOCTYPE HTML><html><head><title>{}</title></head><body><h1>{}</h1><hr><ul>",
            title, title
        );
        let end = "</ul></body></html>";

        let mut page = start;
        let mut entries: Vec<DirEntry> = path
            .read_dir()
            .map_err(FetchError::IOError)?
            .flatten()
            .collect::<Vec<DirEntry>>();
        let cmp = |a: &DirEntry, b: &DirEntry| {
            let check = (a.path().is_dir(), b.path().is_dir());
            match check {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.file_name().cmp(&b.file_name()),
            }
        };
        entries.sort_by(cmp);
        for entry in entries {
            let filename = entry.file_name();
            let filename =
                filename.to_string_lossy() + if entry.path().is_dir() { "/" } else { "" };
            page.push_str(format!("<li><a href={}>{}</a></li>", filename, filename).as_str());
        }
        page.push_str(end);
        Ok(FetchResult::Dir(page))
    } else {
        match File::open(path) {
            Ok(file) => Ok(FetchResult::File(file)),
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => Err(FetchError::FileNotFound),
                _ => Err(FetchError::IOError(e)),
            },
        }
    }
}
