use tokio::sync::oneshot;

#[derive(Debug)]
pub enum ClientCommand {
    Stop,
    Start,
    Disassemble {
        x: usize,
        y: usize,
        respond: oneshot::Sender<Result<String, String>>,
    },
    Observe {
        island_id: usize,
    },
}
