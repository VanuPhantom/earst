# earst

Earst is a Rust library to enable IPC on Unix-like platforms through [FIFO special files](https://man7.org/linux/man-pages/man7/fifo.7.html).

## Usage

### Sending messages to other processes

```rs
use std::io::stdin;
use earst::Sender;

#[tokio::main]
fn main() -> Result<(), std::io::Error> {
    let mut sender = Sender::open("~/my-pipe").await?;
    let input = stdin();

    loop {
        let mut message = String::new();

        input.read_line(&mut message)?;

        sender.send(&message.as_bytes()[..}).await?;
    }
}
```

### Receiving messages from other processes

```rs
use earst::Receiver;

#[tokio::main]
fn main() {
    let mut receiver = Receiver::open("~/my-pipe").await.unwrap();

    loop {
        let message = receiver.receive().await.unwrap();
        let message = String::from_utf8(message).unwrap();

        println!("{}", message);
    }
}
```
