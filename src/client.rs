use crossbeam_channel::{bounded, Receiver, Sender};
use crate::message::Message;
use crate::formatting::print_green;

pub struct Client {
    id: u64,
}

impl Client {
    pub fn new(id: u64) -> Self {
        Client { id }
    }

    pub fn consensus(&self, id: Option<u64>, value: String, tx: Sender<Message>) {
        let message = Message::Consensus(id.unwrap_or(self.id), value);
        print_green(&format!("[Client] CONSENSUS: {:?}", message));
        tx.send(message.clone()).unwrap_or(println!("Failed to send message {:?}", message.clone()));
    }
}
