use std::time::Duration;
use tokio::{net::unix::pipe, time::sleep};
use nix::{
    sys::stat::Mode,
    unistd::mkfifo as mkfifo_internal,
    libc
};

pub type Result<T = ()> = std::io::Result<T>;

pub struct Sender<'a> {
    path: &'a str,
    sender: pipe::Sender
}

fn mkfifo(path: &str) -> Result {
    const FIFO_MODE: Mode = match Mode::from_bits(0o666) {
        Some(mode) => mode,
        None => {
            panic!("Couldn't construct FIFO_MODE.")
        },
    };

    Ok(mkfifo_internal(path, FIFO_MODE)?)
}

impl<'a> Sender<'a> {
    async fn open_sender(path: &str) -> Result<pipe::Sender> {
        loop {
            match pipe::OpenOptions::new().open_sender(path) {
                Ok(sender) => break Ok(sender),
                /* ENXIO = No such device or address
                 * returned whenever there isn't a
                 * receiving end for the pipe */
                Err(error) if error.raw_os_error() == Some(libc::ENXIO) => {
                    sleep(Duration::from_millis(50)).await;
                },
                /* ENOENT = No such file or directory
                 * returned whenever the named pipe
                 * does not exist (yet) */
                Err(error) if error.raw_os_error() == Some(libc::ENOENT) => mkfifo(path)?,
                Err(error) => break Err(error)
            }
        }
    }

    pub async fn open(path: &'a str) -> Result<Self> {
        Ok(Sender {
            path,
            sender: Self::open_sender(path).await?
        })
    }
    pub async fn send(&mut self, data: &[u8]) -> Result {
        let message_length = data.len();
        let header = &message_length.to_le_bytes();
        let message = &[header, data].concat()[..];

        loop {
            match self.sender.try_write(message) {
                Ok(_) => break Ok(()),
                // EPIPE = broken pipe
                Err(error) if error.raw_os_error() == Some(libc::EPIPE) => {
                    self.sender = Self::open_sender(self.path).await?;
                },
                Err(error) => break Err(error)
            }
        }
    }
}

pub struct Receiver<'a> {
    path: &'a str,
    receiver: pipe::Receiver
}

impl<'a> Receiver<'a> {
    async fn open_receiver(path: &str) -> Result<pipe::Receiver> {
        loop {
            match pipe::OpenOptions::new().open_receiver(path) {
                Ok(sender) => break Ok(sender),
                /* ENOENT = No such file or directory
                 * returned whenever the named pipe
                 * does not exist (yet) */
                Err(error) if error.raw_os_error() == Some(libc::ENOENT) => {
                    mkfifo(path)?;
                },
                Err(error) => break Err(error)
            }
        }
    }

    pub async fn open(path: &'a str) -> Result<Self> {
        Ok(Receiver {
            path,
            receiver: Self::open_receiver(path).await?
        })
    }
}
