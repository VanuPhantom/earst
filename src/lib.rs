use std::time::Duration;
use tokio::{net::unix::pipe, time::sleep};
use nix::{
    sys::stat::Mode,
    unistd::mkfifo
};
use libc;

pub struct Sender {
    sender: pipe::Sender
}

impl Sender {
    pub async fn open(path: &str) -> Result<Self, std::io::Error> {
        let sender = loop {
            match pipe::OpenOptions::new().open_sender(path) {
                Ok(sender) => break sender,
                /* ENXIO = No such device or address
                 * returned whenever there isn't a
                 * receiving end for the pipe */
                Err(error) if error.raw_os_error() == Some(libc::ENXIO) => {
                    sleep(Duration::from_millis(50)).await;
                },
                /* ENOENT = No such file or directory
                 * returned whenever the named pipe
                 * does not exist (yet) */
                Err(error) if error.raw_os_error() == Some(libc::ENOENT) => {
                    const FIFO_MODE: Mode = match Mode::from_bits(0o666) {
                        Some(mode) => mode,
                        None => {
                            panic!("Couldn't construct FIFO_MODE.")
                        },
                    };

                    mkfifo(path, FIFO_MODE)?;
                },
                Err(error) => {
                    return Err(error);
                }
            }
        };

        Ok(Sender {
            sender
        })
    }
}
