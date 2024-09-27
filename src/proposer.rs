use crossbeam_channel::{bounded, Receiver, Sender};
use crate::message::Message;
use crate::formatting::print_green; 
use std::sync::{Arc, Mutex};
pub struct Proposer {
    id: u64,
    proposal_number: u64,
    round_number: u64,
}
impl Proposer {
    pub fn new(
        id: u64,
        proposal_number: u64,
    ) -> Self {
        Proposer {
            id,
            proposal_number,
            round_number: 0,
        }
    }

    pub fn handle_consensus(&mut self, tx: &Arc<Mutex<Vec<Sender<Message>>>>, id: Option<u64>, value: String) {
        self.round_number += 1;
        let proposal_number = self.proposal_number + 1 + id.unwrap_or(0);
        let message = Message::Prepare(proposal_number, self.round_number, value);
        for acceptor in tx.lock().unwrap().iter() {
            println!("[Proposer] Sending message: {:?}", message);
            acceptor.send(message.clone()).unwrap();
        }
        self.proposal_number += 1 + id.unwrap_or(0);
    }

    pub fn propose(
        &self,
        proposal_number: u64,
        round_number: u64,
        value: String,
        tx: & Arc<Mutex<Vec<Sender<Message>>>>,
    ) {
        let message = Message::Propose(proposal_number, round_number, value);
        for acceptor in tx.lock().unwrap().iter() {
            println!("[Proposer] Sending message: {:?}", message);
            acceptor.send(message.clone()).unwrap();
        }
    }
}