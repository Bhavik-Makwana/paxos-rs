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
        // self.round_number += 1;
        self.proposal_number += 1 + id.unwrap_or(0);
        let message = Message::Prepare(self.proposal_number, self.id, self.round_number, value);
        for acceptor in tx.lock().unwrap().iter() {
            println!("[Proposer] Sending message: {:?}", message);
            acceptor.send(message.clone()).unwrap();
        }
    }

    pub fn handle_stable_consensus(&mut self, tx: &Arc<Mutex<Vec<Sender<Message>>>>, id: Option<u64>, value: String) {
        let proposal_number = self.proposal_number + 1 + id.unwrap_or(0);
        let message = Message::Propose(proposal_number, self.id, self.round_number, value);
        for acceptor in tx.lock().unwrap().iter() {
            println!("[Proposer] Sending message: {:?}", message);
            acceptor.send(message.clone()).unwrap();
        }
    }

    pub fn update_round_number(&mut self, round_number: u64, tx: &Arc<Mutex<Vec<Sender<Message>>>>) {
        self.round_number = round_number;
        self.reset_acceptors(tx);
    }

    pub fn reset_acceptors(&mut self, tx: &Arc<Mutex<Vec<Sender<Message>>>>) {
        for acceptor in tx.lock().unwrap().iter() {
            acceptor.send(Message::Reset).unwrap();
        }
    }

    pub fn propose(
        &self,
        proposal_number: u64,
        round_number: u64,
        value: String,
        tx: & Arc<Mutex<Vec<Sender<Message>>>>,
    ) {
        let message = Message::Propose(proposal_number, self.id, round_number, value);
        for acceptor in tx.lock().unwrap().iter() {
            println!("[Proposer] Sending message: {:?}", message);
            acceptor.send(message.clone()).unwrap();
        }
    }
}