use crossbeam_channel::{bounded, Receiver, Sender};
use crate::message::Message;
use crate::formatting::{print_green, print_red};
use std::sync::{Arc, Mutex};
pub struct Client {
    id: u64,
    leader_id: Option<u64>,
}

impl Client {
    pub fn new(id: u64) -> Self {
        Client { id, leader_id: None }
    }

    pub fn consensus(&self, id: Option<u64>, value: String, tx: Sender<Message>) {
        let message = Message::Consensus(id.unwrap_or(0), value);
        print_green(&format!("[Client] CONSENSUS: {:?}", message));
        tx.send(message.clone()).unwrap_or(println!("Failed to send message {:?}", message.clone()));
    }

    pub fn send_to_stable_leader(&self, id: Option<u64>, value: String, tx: Sender<Message>) {
        let message = Message::StableConsensus(id.unwrap_or(0), value);
        print_green(&format!("[Client] CONSENSUS: {:?}", message));
        tx.send(message.clone()).unwrap_or(println!("Failed to send message {:?}", message.clone()));
    }

    pub fn await_stable_leader(&mut self, rxs: Arc<Mutex<Vec<Receiver<Message>>>>) {
        let message = rxs.lock().unwrap()[self.id as usize].recv().unwrap();
        match message {
            Message::LeaderID(id) => {
                print_green(&format!("[Client] LEADER ID: {:?}", id));
                self.leader_id = Some(id);
            }
            _ => {
                print_red(&format!("[Client] Received unexpected message: {:?}", message));
            }
        }
    }
}
