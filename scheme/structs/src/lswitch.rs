use tokio::sync::mpsc;

const TOKIO_CHANNEL_SIZE: usize = 10;

#[derive(Eq, PartialEq, Debug)]
pub enum InterruptSignal {
    Interrupted,
    NInterrupted,
}

#[derive(Clone, Debug)]
pub struct InterruptionSender {
    inner: mpsc::Sender<InterruptSignal>,
}

impl InterruptionSender {
    pub fn new(tx: mpsc::Sender<InterruptSignal>) -> Self {
        Self { inner: tx }
    }

    pub async fn interrupt(&mut self) {
        let _ = self.inner.try_send(InterruptSignal::Interrupted);
    }
}

pub struct InterruptionReceiver {
    inner: mpsc::Receiver<InterruptSignal>,
}

impl InterruptionReceiver {
    pub fn new(rx: mpsc::Receiver<InterruptSignal>) -> Self {
        Self { inner: rx }
    }

    pub fn is_interrupted(&mut self) -> InterruptSignal {
        match self.inner.try_recv() {
            Ok(i) => i,
            Err(_) => InterruptSignal::NInterrupted,
        }
    }

    pub async fn recv(&mut self) -> Option<InterruptSignal> {
        self.inner.recv().await
    }
}

pub fn new_interruption_handler() -> (InterruptionSender, InterruptionReceiver) {
    let (tx, rx) = mpsc::channel(TOKIO_CHANNEL_SIZE);

    let tx = InterruptionSender::new(tx);
    let rx = InterruptionReceiver::new(rx);
    (tx, rx)
}