use socket2::{Domain, SockAddr, Socket, Type};
use std::{
    env::{self, consts::OS},
    error::Error,
    fs::File,
    io::{Read, Write},
    net::Shutdown,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct Discord {
    client_id: String,
    socket_file: SocketFile,
}

impl Discord {
    pub fn new(client_id: &String) -> Result<Discord, Box<dyn Error>> {
        let socket_file = Discord::new_socket_file()?;
        Ok(Discord {
            client_id: client_id.clone(),
            socket_file,
        })
    }

    #[cfg(target_os = "windows")]
    fn new_socket_file() -> Result<SocketFile, Box<dyn Error>> {
        Ok(SocketFile::File(File::open(Discord::get_ipc_path())?))
    }

    #[cfg(not(target_os = "windows"))]
    fn new_socket_file() -> Result<SocketFile, Box<dyn Error>> {
        let socket = Socket::new(Domain::unix(), Type::stream(), None)?;
        let socket_addr = SockAddr::unix(Discord::get_ipc_path())?;
        socket.connect(&socket_addr)?;
        Ok(SocketFile::Socket(socket))
    }

    fn get_ipc_path() -> PathBuf {
        match OS {
            "windows" => Path::new("\\\\?\\pipe\\discord-ipc-0").to_path_buf(),
            _ => {
                let mut ipc_path = env::temp_dir();
                ipc_path.push("discord-ipc-0");
                ipc_path
            }
        }
    }

    pub fn send(&mut self, opcode: i32, payload: &String) -> Result<(), Box<dyn Error>> {
        let msg = Discord::encode(opcode, payload)?;
        match self.socket_file {
            SocketFile::Socket(ref socket) => {
                socket.send(&msg[..])?;
            }
            SocketFile::File(ref mut file) => {
                file.write(&msg[..])?;
                let _ = file.flush()?;
            }
        }
        Ok(())
    }

    pub fn recv(&mut self) -> Result<String, Box<dyn Error>> {
        match self.socket_file {
            SocketFile::Socket(ref socket) => {
                let mut response: [u8; 2048] = [0; 2048];
                socket.recv(&mut response)?;
                Ok(String::from_utf8_lossy(&response[8..])
                    .trim_matches(char::from(0))
                    .to_string())
            }
            SocketFile::File(ref mut file) => {
                let mut header: [u8; 8] = [0; 8];
                let mut header_size = 8;
                while header_size > 0 {
                    header_size -= file.read(&mut header)?;
                }

                let header: (i32, i32) = Packer::unpack(header);
                let mut response: [u8; 2048] = [0; 2048];
                let mut response_size = header.1 as usize;
                while response_size > 0 {
                    response_size -= file.read(&mut response)?;
                }

                Ok(String::from_utf8_lossy(&response[..])
                    .trim_matches(char::from(0))
                    .to_string())
            }
        }
    }

    fn encode(opcode: i32, payload: &String) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut msg = Vec::new();
        msg.append(&mut Packer::pack((opcode, payload.len() as i32)).to_vec());
        msg.append(&mut payload.as_bytes().to_vec());
        Ok(msg)
    }

    pub fn handshake(&mut self) -> Result<String, Box<dyn Error>> {
        let handshake = format!(r#"{{ "v": 1, "client_id": "{}" }}"#, self.client_id);
        self.send(0, &handshake)?;
        Ok(self.recv()?)
    }

    pub fn command(&mut self, payload: &String) -> Result<String, Box<dyn Error>> {
        self.send(1, payload)?;
        self.recv()
    }

    pub fn disconnect(mut self) -> Result<(), Box<dyn Error>> {
        self.send(2, &r#"{}"#.to_string())?;
        match self.socket_file {
            SocketFile::Socket(socket) => {
                socket.shutdown(Shutdown::Both)?;
            }
            _ => {}
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum SocketFile {
    Socket(Socket),
    File(File),
}

#[derive(restruct_derive::Struct)]
#[fmt = "<ii"]
struct Packer;
