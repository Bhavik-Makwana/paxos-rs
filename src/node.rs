use crossbeam_channel::{bounded, Receiver, Sender};
use crate::message::Message;
use crate::formatting::print_green; 
use std::sync::{Arc, Mutex};
use crate::formatting::print_red;

pub struct Node {
    id: u64,
    proposal_number: u64,
    round_number: u64,
    max_id: u64,
    proposal_accepted: bool,
    accepted_value: Option<String>,
    accepted_proposal_number: Option<u64>,
}
impl Node {
    pub fn new(
        id: u64,
        proposal_number: u64,
    ) -> Self {
        Node {
            id,
            proposal_number,
            round_number: 0,
            max_id: 0,
            proposal_accepted: false,
            accepted_value: None,
            accepted_proposal_number: None,
        }
    }

    // Node
    pub fn handle_consensus(&mut self, tx: &Arc<Mutex<Vec<Sender<Message>>>>, id: Option<u64>, value: String) {
        // self.round_number += 1;
        self.proposal_number += 1 + id.unwrap_or(0);
        let message = Message::Prepare(self.proposal_number, self.id, self.round_number, value);
        for acceptor in tx.lock().unwrap().iter() {
            println!("[Node] Sending message: {:?}", message);
            acceptor.send(message.clone()).unwrap();
        }
    }

    pub fn handle_stable_consensus(&mut self, tx: &Arc<Mutex<Vec<Sender<Message>>>>, id: Option<u64>, value: String) {
        let proposal_number = self.proposal_number + 1 + id.unwrap_or(0);
        let message = Message::Propose(proposal_number, self.id, self.round_number, value);
        for acceptor in tx.lock().unwrap().iter() {
            println!("[Node] Sending message: {:?}", message);
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
            println!("[Node] Sending message: {:?}", message);
            acceptor.send(message.clone()).unwrap();
        }
    }

    // ACCEPTOR
    pub fn handle_prepare(&mut self, proposal_number: u64, leader_id: u64, round_number: u64, value: String, tx: &Arc<Mutex<Vec<Sender<Message>>>>) {
        if proposal_number <= self.max_id {
            print_green(&format!("[Acceptor] PREPARE SEND FAIL: {:?}", Message::Fail(value.clone())));
            for node in tx.lock().unwrap().iter() {
                node.send(Message::Fail(value.clone())).unwrap();
            }
        } else {
            self.max_id = proposal_number;
            if self.proposal_accepted {
                let message = Message::Promise(proposal_number, leader_id, round_number, self.accepted_proposal_number, self.accepted_value.clone().unwrap());
                print_green(&format!("[Acceptor] SEND ACCEPTED PROMISE: {:?}", message));
                for node in tx.lock().unwrap().iter() {
                    node.send(message.clone()).unwrap();
                }
            } else {
                let message = Message::Promise(proposal_number, leader_id, round_number, None, value.clone());
                print_green(&format!("[Acceptor] SEND PROMISE: {:?}", message));
                for node in tx.lock().unwrap().iter() {
                    node.send(message.clone()).unwrap();
                }
            }
        }
    }
    
    pub fn reset(&mut self) {
        self.proposal_accepted = false;
        self.accepted_value = None;
        self.accepted_proposal_number = None;
    }
    
    pub fn handle_propose(&mut self, proposal_number: u64, leader_id: u64, round_number: u64, value: String, tx: &Arc<Mutex<Vec<Sender<Message>>>>) {
        if proposal_number >= self.max_id {
            self.max_id = proposal_number;
            self.proposal_accepted = true;
            self.accepted_value = Some(value.clone());
            self.accepted_proposal_number = Some(proposal_number);
            // if self.round_number < round_number {
            //     self.accepted_proposal_number = None;
            //     self.accepted_value = None;
            //     self.proposal_accepted = false;
            //     self.round_number = round_number;
            // }
            for node in tx.lock().unwrap().iter() {
                node.send(Message::Accept(proposal_number, leader_id, round_number, value.clone())).unwrap();
            }
        } else {
            for node in tx.lock().unwrap().iter() {
                node.send(Message::Fail(value.clone())).unwrap();
            }
        }
    }
    
    // LEARNER
    pub fn record(&self, proposal_number: u64, round_number: u64, value: String, storage: &mut Arc<Mutex<Vec<(u64, String)>>>, txs: &Arc<Mutex<Vec<Sender<Message>>>>, client_txs: &Arc<Mutex<Vec<Sender<Message>>>>, leader_id: u64) {
        print_red(&format!("[Learner] Recording value: {:?} with proposal number: {:?} and round number: {:?}", value, proposal_number, round_number));
        let mut storage_guard: std::sync::MutexGuard<'_, Vec<(u64, String)>> = storage.lock().unwrap();
        if !storage_guard.contains(&(round_number, value.clone())) {
            storage_guard.push((round_number, value.clone()));
            self.update_round_number_learner(round_number, &txs);
            self.send_stable_leader(client_txs, leader_id);
        }
    }

    fn update_round_number_learner(&self, round_number: u64, txs: &Arc<Mutex<Vec<Sender<Message>>>>) {
        for acceptor in txs.lock().unwrap().iter() {
            acceptor.send(Message::RoundNumber(round_number+1)).unwrap();
        }
    }

    fn send_stable_leader(&self, client_txs: &Arc<Mutex<Vec<Sender<Message>>>>, leader_id: u64) {
        for client_tx in client_txs.lock().unwrap().iter() {
            client_tx.send(Message::LeaderID(leader_id)).unwrap();
        }
    }
}