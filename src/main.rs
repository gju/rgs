use std::fs::{self, DirEntry, File};
use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;

const CRLF: &str = "\r\n";
const MAP_FILE_NAME: &str = "map.txt";

struct GopherServer {
    host: String,
    port: String,
    root: String,
}

impl GopherServer {
    fn generate_map_file(&self, path: &Path, file_name: &str) -> io::Result<()> {
        let dir_entries: Vec<DirEntry> = fs::read_dir(&path)?
            .filter_map(|entry| entry.ok())
            .collect();
        let map_file_path = path.join(file_name);
        let mut map_file = File::create(map_file_path)?;

        for entry in dir_entries {
            let file_type = entry.file_type().expect("could not read file type");
            let item_prefix = if file_type.is_dir() { "1" } else { "0" };
            writeln!(
                map_file,
                "{}{}\t{}\t{}\t{}",
                item_prefix,
                entry.file_name().to_str().unwrap(),
                entry.path().to_str().unwrap(),
                self.host,
                self.port,
            )?;
        }
        writeln!(map_file, ".")?;

        Ok(())
    }

    fn serve_map(&self, path: &Path, stream: &mut TcpStream) -> io::Result<()> {
        println!("requested map file for {:?}", path);
        let map_file_path = path.join(MAP_FILE_NAME);
        if !map_file_path.exists() {
            println!("generating map file for {:?}", path);
            self.generate_map_file(path, MAP_FILE_NAME)?;
        }
        self.serve_file(&map_file_path, stream)?;

        Ok(())
    }

    fn serve_file(&self, path: &Path, stream: &mut TcpStream) -> io::Result<()> {
        println!("serving file: {}", path.to_str().unwrap());
        let mut file = File::open(path)?;
        io::copy(&mut file, stream)?;

        Ok(())
    }

    fn serve_path(&self, selector: &str, stream: &mut TcpStream) -> io::Result<()> {
        let path = Path::new(selector.trim());

        if path.is_dir() {
            self.serve_map(path, stream)?;
        } else {
            self.serve_file(path, stream)?;
        }

        Ok(())
    }

    fn handle_stream(&self, mut stream: TcpStream) -> io::Result<()> {
        let mut reader = BufReader::new(&stream);
        let mut buf = String::new();
        reader.read_line(&mut buf)?;

        match buf.as_str() {
            CRLF => self.serve_map(Path::new(self.root.as_str()), &mut stream)?,
            selector => self.serve_path(selector, &mut stream)?,
        };

        Ok(())
    }

    fn start(&self) -> io::Result<()> {
        let addr = format!("{}:{}", self.host, self.port);
        let listener = TcpListener::bind(addr)?;

        for stream in listener.incoming() {
            if let Err(err) = self.handle_stream(stream?) {
                eprintln!("error handling request: {:?}", err);
            }
        }

        Ok(())
    }
}

fn main() -> io::Result<()> {
    let server = GopherServer {
        host: String::from("localhost"),
        port: String::from("7070"),
        root: String::from("."),
    };

    server.start()?;

    Ok(())
}
