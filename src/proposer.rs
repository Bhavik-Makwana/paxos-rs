use crossbeam_channel::{bounded, Receiver, Sender};
use crate::message::Message;
use crate::formatting::print_green; 
use std::sync::{Arc, Mutex};
pub struct Proposer<'a> {
    id: u64,
    proposal_number: u64,
    proposal_value: String,
    acceptors: Vec<u64>,
    responses: Vec<Option<(u64, String)>>,
    rx: &'a Receiver<Message>,
    tx: &'a Sender<Message>,
}
impl<'a> Proposer<'a> {
    pub fn new(
        id: u64,
        proposal_number: u64,
        proposal_value: String,
        acceptors: Vec<u64>,
        responses: Vec<Option<(u64, String)>>,
        rx: &'a Receiver<Message>,
        tx: &'a Sender<Message>,
    ) -> Self {
        Proposer {
            id,
            proposal_number,
            proposal_value,
            acceptors,
            responses,
            rx,
            tx,
        }
    }

    pub fn handle_consensus(&mut self, tx: &'a Arc<Mutex<Vec<Sender<Message>>>>, id: Option<u64>, value: String) {
        let proposal_number = self.proposal_number + 1 + id.unwrap_or(0);
        let message = Message::Prepare(proposal_number, value);
        for acceptor in tx.lock().unwrap().iter() {
            println!("[Proposer] Sending message: {:?}", message);
            acceptor.send(message.clone()).unwrap();
        }
        self.proposal_number += 1 + id.unwrap_or(0);
    }

    pub fn propose(
        &self,
        proposal_number: u64,
        value: String,
        tx: &'a Arc<Mutex<Vec<Sender<Message>>>>,
    ) {
        let message = Message::Propose(proposal_number, value);
        for acceptor in tx.lock().unwrap().iter() {
            println!("[Proposer] Sending message: {:?}", message);
            acceptor.send(message.clone()).unwrap();
        }
    }
}