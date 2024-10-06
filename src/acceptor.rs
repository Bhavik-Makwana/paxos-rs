use crossbeam_channel::{Receiver, Sender};
use crate::message::Message;
use crate::formatting::print_green;

pub struct Acceptor {
    id: u64,
    max_id: u64,
    proposal_accepted: bool,
    accepted_value: Option<String>,
    accepted_proposal_number: Option<u64>,
    round_number: u64,
}

impl Acceptor {
    pub fn new(id: u64, max_id: u64) -> Self {
        Acceptor {
            id,
            max_id,
            proposal_accepted: false,
            accepted_value: None,
            accepted_proposal_number: None,
            round_number: 0,
        }
    }

    pub fn handle_prepare(&mut self, proposal_number: u64, leader_id: u64, round_number: u64, value: String, tx: &Sender<Message>) {
        if proposal_number <= self.max_id {
            print_green(&format!("[Acceptor] PREPARE SEND FAIL: {:?}", Message::Fail(value.clone())));
            tx.send(Message::Fail(value.clone())).unwrap();
        } else {
            self.max_id = proposal_number;
            if self.proposal_accepted {
                let message = Message::Promise(proposal_number, leader_id, round_number, self.accepted_proposal_number, self.accepted_value.clone().unwrap());
                print_green(&format!("[Acceptor] SEND ACCEPTED PROMISE: {:?}", message));
                tx.send(message).unwrap();
            } else {
                let message = Message::Promise(proposal_number, leader_id, round_number, None, value.clone());
                print_green(&format!("[Acceptor] SEND PROMISE: {:?}", message));
                tx.send(message).unwrap();
            }
        }
    }
    
    pub fn reset(&mut self) {
        self.proposal_accepted = false;
        self.accepted_value = None;
        self.accepted_proposal_number = None;
    }
    
    pub fn handle_propose(&mut self, proposal_number: u64, leader_id: u64, round_number: u64, value: String, tx: &Sender<Message>) {
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
            tx.send(Message::Accept(proposal_number, leader_id, round_number, value.clone())).unwrap();
        } else {
            tx.send(Message::Fail(value.clone())).unwrap();
        }
    }
}