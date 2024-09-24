use crossbeam_channel::{bounded, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
mod learner;
use learner::Learner;
mod message;
mod client;
mod proposer;
mod acceptor;
mod formatting;
use message::Message;
use client::Client;
use proposer::Proposer;
use acceptor::Acceptor;
use formatting::{print_green, print_red};

const NUM_PROPOSERS: usize = 1;
const NUM_ACCEPTORS: usize = 3;
const NUM_LEARNERS: usize = 1;

fn setup_channels(nodes: usize) -> (Arc<Mutex<Vec<Sender<Message>>>>, Arc<Mutex<Vec<Receiver<Message>>>>) {

    let mut proposer_txs = vec![];
    let mut learner_rxs = vec![];
    for _ in 0..nodes {
        let (tx, rx) = bounded(100);
        learner_rxs.push(rx.clone());
        proposer_txs.push(tx.clone());
    }
    (Arc::new(Mutex::new(proposer_txs)), Arc::new(Mutex::new(learner_rxs)))
}


fn setup_learners(learners: &mut Vec<thread::JoinHandle<()>>, learner_rxs: Arc<Mutex<Vec<Receiver<Message>>>>, storage: Arc<Mutex<Vec<String>>>) {
    for i in 0..NUM_LEARNERS {
        let learner_rx_binding = learner_rxs.lock().unwrap()[i].clone();
        let mut storage_binding = storage.clone();
        let handle = thread::spawn(move || {
            let learner = Learner::new(i as u64, &learner_rx_binding);
            loop {
                let message = learner_rx_binding.recv().unwrap();
                match message {
                    Message::Accept(proposal_number, value) => {
                        learner.record(proposal_number, value, &mut storage_binding);
                    }
                    Message::Terminate => {
                        println!("[Learner] Received TERMINATE");
                        break;
                    }
                    _ => {
                        print_red(&format!("[Learner] Received message: {:?}", message));
                        panic!("[Learner] Received message: {:?}", message);
                    }
                }
            }
        });
        learners.push(handle);
    }
}
fn setup_proposers(proposers: &mut Vec<thread::JoinHandle<()>>, 
proposer_txs: &mut Vec<Sender<Message>>, 
acceptor_txs: Arc<Mutex<Vec<Sender<Message>>>>, 
learner_txs: Arc<Mutex<Vec<Sender<Message>>>>) {
    for i in 0..NUM_PROPOSERS {
        let (tx, rx) = bounded(100);
        let acceptor_txs_binding = acceptor_txs.clone();
        let learner_txs_binding = learner_txs.clone();
        proposer_txs.push(tx.clone()); // Store the tx channel
        let handle = thread::spawn(move || {
            let mut proposer = Proposer::new(i as u64, 0, "values".to_string(), vec![], vec![], &rx, &tx);
            // proposer.run();
            let mut proposals = vec![];
            let mut accepted = 0;
            loop {
                let message = rx.recv().unwrap();
                match message {
                    Message::Consensus(_, value) => {
                        // println!("Received consensus message: {:?}", value);
                        proposer.handle_consensus(&acceptor_txs_binding, value);
                    }
                    Message::Promise(proposal_number, accepted_proposal_number, value) => {
                        // println!("PROMISE: {:?}", proposal_number);
                        proposals.push((proposal_number, accepted_proposal_number, value));
                        // println!("proposals: {:?}", proposals);
                        if proposals.len() >= (NUM_ACCEPTORS / 2) + 1 {
                            println!("[Proposer] Achieved quorum");
                            let all_same = proposals.iter().all(|&(_, _, ref v)| v == &proposals[0].2);
                            if all_same {
                                println!("[Proposer] All proposals are the same {:?}", proposals);
                                let value = proposals[0].2.clone();
                                if let Some(accepted_proposal_number) = proposals[0].1 {
                                    proposer.propose(accepted_proposal_number, value, &acceptor_txs_binding);
                                } else {
                                    proposer.propose(proposals[0].0, value, &acceptor_txs_binding);
                                }
                                proposals.clear();
                            } else {
                                println!("[Proposer] Proposals are not the same");
                                let value = proposals
                                .iter()
                                .max_by_key(|proposal| proposal.0)
                                .unwrap()
                                .clone();
                                proposer.propose(value.1.unwrap(), value.2, &acceptor_txs_binding);
                                proposals.clear();
                            }
                            
                        } 
                        // else {
                        //     println!("[Proposer] Did not achieve quorum");
                        //     // proposals.clear();
                        // }
                    }
                    Message::Accept(proposal_number, value) => {
                        println!("[Proposer] Received ACCEPT: {:?}", value);
                        accepted += 1;
                        if accepted >= (NUM_ACCEPTORS / 2) + 1 {
                            println!("[Proposer] ACCEPT QUORUM REACHED");
                            accepted = 0;
                            for learner_tx in learner_txs_binding.lock().unwrap().iter() {
                                learner_tx
                                    .send(Message::Accept(proposal_number, value.clone()))
                                    .unwrap();
                            }
                        }
                    }
                    Message::Fail => {
                        println!("[Proposer] Received FAIL");
                    }
                    Message::Terminate => {
                        println!("[Proposer] Received TERMINATE");
                        break;
                    }
                    _ => {
                        println!("[Proposer] Received message: {:?}", message);
                        panic!("[Proposer] Received message: {:?}", message);
                    }
                }
                // thread::sleep(Duration::from_secs(1));
            }
        });
        proposers.push(handle);
    }
}

fn setup_acceptors(acceptors: &mut Vec<thread::JoinHandle<()>>, 
acceptor_rxs: Arc<Mutex<Vec<Receiver<Message>>>>, 
acceptor_txs: Arc<Mutex<Vec<Sender<Message>>>>, 
proposer_txs: Vec<Sender<Message>>) {
    for i in 0..NUM_ACCEPTORS as usize {
        // let (tx, rx) = bounded(100);
        let rx_binding = acceptor_rxs.lock().unwrap()[i].clone();
        let tx_binding = acceptor_txs.lock().unwrap()[i].clone();
        let proposer_tx_binding = proposer_txs[0].clone();
        let handle = thread::spawn(move || {
            let mut acceptor = Acceptor::new(i as u64, 0);
            loop {
                let message = rx_binding.recv().unwrap();
                match message {
                    Message::Prepare(proposal_number, value) => {
                        acceptor.handle_prepare(proposal_number, value, &proposer_tx_binding);
                    }
                    Message::Propose(proposal_number, value) => {
                        acceptor.handle_propose(proposal_number, value, &proposer_tx_binding);
                    }
                    Message::Fail => {
                        println!("[Acceptor] Received FAIL");
                    }
                    Message::Terminate => {
                        println!("[Acceptor] Received TERMINATE");
                        break;
                    }
                    _ => {
                        println!("\x1b[31m[Acceptor] Received message: {:?}\x1b[0m", message);
                        panic!("\x1b[31m[Acceptor] Received message: {:?}\x1b[0m", message);
                    }
                }
                // thread::sleep(Duration::from_secs(1));
            }
        });
        acceptors.push(handle);
    }
}


fn main() {
  
    let client = Client::new(0);
    let mut storage = Arc::new(Mutex::new(vec![]));
    // Nodes
    let mut proposers    = vec![];
    let mut acceptors = vec![];
    let mut learners = vec![];
    
    // Channels
    let mut proposer_txs = vec![]; // Store multiple tx channels
    let (learner_txs, learner_rxs) = setup_channels(NUM_LEARNERS);
    let (acceptor_txs, acceptor_rxs) = setup_channels(NUM_ACCEPTORS);
    // PROPOSERS
    setup_proposers(&mut proposers, &mut proposer_txs, acceptor_txs.clone(), learner_txs);

    // ACCEPTORS
    setup_acceptors(&mut acceptors, acceptor_rxs.clone(), acceptor_txs.clone(), proposer_txs.clone());
    
    // LEARNERS
    setup_learners(&mut learners, learner_rxs.clone(), storage.clone());

    client.consensus("values".to_string(), proposer_txs[0].clone());
    client.consensus("wabbit".to_string(), proposer_txs[0].clone());
    client.consensus("wabb2it".to_string(), proposer_txs[0].clone());
    // client.consensus("values3".to_string(), proposer_txs[0].clone());

    for proposer in proposers {
        proposer.join().unwrap();
    }
    for acceptor in acceptors {
        acceptor.join().unwrap();
    }
    for learner in learners {
        learner.join().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam_channel::unbounded;
    use std::sync::{Arc, Mutex};


    #[test]
    fn test_propose_single_value() {
        let client = Client::new(0);

        // Nodes
        let mut proposers    = vec![];
        let mut acceptors = vec![];
        let mut learners = vec![];
        let mut storage = Arc::new(Mutex::new(vec![]));
        // Channels
        let mut proposer_txs = vec![]; // Store multiple tx channels
        let (learner_txs, learner_rxs) = setup_channels(NUM_LEARNERS);
        let (acceptor_txs, acceptor_rxs) = setup_channels(NUM_ACCEPTORS);
        // PROPOSERS
        setup_proposers(&mut proposers, &mut proposer_txs, acceptor_txs.clone(), learner_txs.clone());

        // ACCEPTORS
        setup_acceptors(&mut acceptors, acceptor_rxs.clone(), acceptor_txs.clone(), proposer_txs.clone());
        
        // LEARNERS
        setup_learners(&mut learners, learner_rxs.clone(), storage.clone());

        client.consensus("values".to_string(), proposer_txs[0].clone());
        thread::sleep(Duration::from_secs(2));
        for proposer in proposer_txs.iter() {
            proposer.send(Message::Terminate).unwrap();
        }
        for acceptor in acceptor_txs.lock().unwrap().iter() {
            acceptor.send(Message::Terminate).unwrap();
        }
        for learner in learner_txs.lock().unwrap().iter() {
            learner.send(Message::Terminate).unwrap();
        }

        for proposer in proposers {
            proposer.join().unwrap();
        }
        for acceptor in acceptors {
            acceptor.join().unwrap();
        }
        for learner in learners {
            learner.join().unwrap();
        }
        println!("storage: {:?}", storage.lock().unwrap());
        assert_eq!(storage.lock().unwrap().len(), 1);
    }
}
