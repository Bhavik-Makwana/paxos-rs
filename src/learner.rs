use crossbeam_channel::{bounded, Receiver, Sender};
use crate::message::Message;
use crate::formatting::print_red;
use std::sync::{Arc, Mutex};
pub struct Learner {
    id: u64,
}
impl Learner {
    pub fn new(id: u64) -> Self {
        Learner { id }
    }

    pub fn record(&self, proposal_number: u64, round_number: u64, value: String, storage: &mut Arc<Mutex<Vec<(u64, String)>>>, txs: &Vec<Sender<Message>>) {
        print_red(&format!("[Learner] Recording value: {:?} with proposal number: {:?} and round number: {:?}", value, proposal_number, round_number));
        let mut storage_guard: std::sync::MutexGuard<'_, Vec<(u64, String)>> = storage.lock().unwrap();
        if !storage_guard.contains(&(round_number, value.clone())) {
            storage_guard.push((round_number, value.clone()));
            self.update_round_number(round_number, &txs);
        }
    }

    fn update_round_number(&self, round_number: u64, txs: &Vec<Sender<Message>>) {
        for acceptor in txs.iter() {
            acceptor.send(Message::RoundNumber(round_number+1)).unwrap();
        }
    }
}
