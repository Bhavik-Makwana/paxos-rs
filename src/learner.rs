use crossbeam_channel::{bounded, Receiver, Sender};
use crate::message::Message;
use crate::formatting::print_red;
use std::sync::{Arc, Mutex};
pub struct Learner<'a> {
    id: u64,
    rx: &'a Receiver<Message>,
}
impl<'a> Learner<'a> {
    pub fn new(id: u64, rx: &'a Receiver<Message>) -> Self {
        Learner { id, rx }
    }

    pub fn record(&self, proposal_number: u64, value: String, storage: &mut Arc<Mutex<Vec<String>>>) {
        print_red(&format!("[Learner] Recording value: {:?} with proposal number: {:?}", value, proposal_number));
        storage.lock().unwrap().push(value);
    }
}
