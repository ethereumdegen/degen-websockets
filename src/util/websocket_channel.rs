use tokio::sync::mpsc::{ Receiver, Sender, channel};

pub struct TokioMpscChannel<T> {
    pub rx: Option< Receiver<T> >,
    pub tx: Sender<T>
}

impl<T> TokioMpscChannel<T> {
    pub fn new(size: usize) -> Self {
        let (tx, rx): (Sender<T>, Receiver<T>) = channel(size);

        Self {
            rx:Some(rx),
            tx
        }
    }
}
