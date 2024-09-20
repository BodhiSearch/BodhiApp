use tokio::sync::mpsc::{Receiver, Sender};

pub fn test_channel() -> (Sender<String>, Receiver<String>) {
  tokio::sync::mpsc::channel::<String>(100)
}
