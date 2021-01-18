use httparse::Request;
use std::fs::DirEntry;
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

pub fn handle_connection(mut stream: &TcpStream, directory: String) -> Result<(), HttpError> {
    let mut buf = [0; 1024];
    stream.read(&mut buf).map_err(HttpError::FailedRead)?;

    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = Request::new(&mut headers);
    let _status = req.parse(&buf).map_err(HttpError::FailedParse)?;

    handle_request(&req, &stream, &directory)?;

    Ok(())
}

fn handle_request(req: &Request, mut stream: &TcpStream, directory: &str) -> Result<(), HttpError> {
    let (version, method, path) = all_fields(&req)?;
    println!(
        "Request:\n{} {} {} from {:?}",
        version,
        method,
        path,
        stream.peer_addr()
    );

    let response = match fetch_path(path, directory) {
        Ok(res) => {
            let mut v = format!(
                "HTTP/1.1 200 Ok\n\
                Content-Type: text/html; charset=utf-8\n\
                {}\n\n",
                if !res.is_dir {
                    "Content-Disposition: attachment\n\n"
                } else {
                    ""
                }
            )
            .into_bytes();
            v.extend(res.data);
            v
        }
        Err(FetchError::FileNotFound) => String::from(
            "HTTP/1.1 404 Not Found\n\
        Content-Type: text/html; charset=utf-8\n\n\
        <h1> Error: File Not Found",
        )
        .into_bytes(),
        Err(FetchError::IOError(_)) => String::from(
            "HTTP/1.1 500 Server Error\n\
            Content-Type: text/html; charset=utf-8\n\n\
            <h1> 500 Intenal Error",
        )
        .into_bytes(),
    };

    stream
        .write(response.as_slice())
        .map_err(HttpError::FailedWrite)?;
    Ok(())
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

pub enum FetchError {
    FileNotFound,
    IOError(std::io::Error),
}

pub struct FetchResult {
    pub is_dir: bool,
    pub data: Vec<u8>,
}

pub fn fetch_path(path_str: &str, directory: &str) -> Result<FetchResult, FetchError> {
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
        let mut data = Vec::new();
        data.extend_from_slice(page.as_bytes());
        Ok(FetchResult { data, is_dir: true })
    } else {
        match std::fs::read(path) {
            Ok(file) => {
                let mut data = Vec::new();
                data.extend_from_slice(file.as_slice());
                Ok(FetchResult {
                    data,
                    is_dir: false,
                })
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => Err(FetchError::FileNotFound),
                _ => Err(FetchError::IOError(e)),
            },
        }
    }
}
