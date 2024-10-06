use crossbeam_channel::{bounded, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
mod message;
mod client;
mod formatting;
mod node;
use message::Message;
use client::Client;
use formatting::{print_green, print_red};
use node::Node;
const NUM_ACCEPTORS: usize = 3;
const NUM_CLIENTS: usize = 2;
const NUM_NODES: usize = 3;

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


fn setup_nodes(nodes:&mut Vec<thread::JoinHandle<()>>, storage: &mut Arc<Mutex<Vec<(u64, String)>>>, client_txns: Arc<Mutex<Vec<Sender<Message>>>>, txns: Arc<Mutex<Vec<Sender<Message>>>>, rxns: Arc<Mutex<Vec<Receiver<Message>>>>) {
    for i in 0..NUM_NODES {
        let rx = rxns.lock().unwrap()[i].clone();
        let txns_binding = txns.clone();
        let mut storage_binding = storage.clone();
        let client_tx_binding = client_txns.clone();
        let mut proposals = vec![];
        let mut accepted_values = vec![];
        let handle = thread::spawn(move || {
            let mut node = Node::new(i as u64, 0);
            loop {
                let message = rx.recv().unwrap();
                match message {
                    // LEARNER
                    Message::Accept(proposal_number, leader_id, round_number, value) => {
                        node.record(proposal_number, round_number, value.clone(), &mut storage_binding, &txns_binding, &client_tx_binding, leader_id);
                    }
                    Message::Terminate => {
                        println!("[Learner] Received TERMINATE");
                        break;
                    }
                    // PROPOSER
                    Message::Consensus(id, value) => {
                        node.handle_consensus(&txns_binding, Some(id), value);
                    }
                    Message::StableConsensus(id, value) => {
                        node.handle_stable_consensus(&txns_binding, Some(id), value);
                    }
                    Message::Promise(proposal_number, leader_id, round_number, accepted_proposal_number, value) => {
                        println!("[Proposer] Received PROMISE: {:?}", value);
                        proposals.push((proposal_number, accepted_proposal_number, value));
                        println!("[Proposer] Proposals: {:?}", proposals);
                        if proposals.len() >= (NUM_ACCEPTORS / 2) + 1 {
                            println!("[Proposer] Achieved quorum");
                            let contains_accepted_value = proposals.iter().any(|&(_, ref accepted_proposal_number, _)| accepted_proposal_number.is_some());
                            let propose_value;
                            if contains_accepted_value {
                                propose_value = proposals
                                .iter()
                                .max_by_key(|proposal| proposal.1.unwrap_or(0))
                                .unwrap()
                                .2.clone();
                            } else {
                                propose_value = proposals[0].2.clone();
                            }
                            node.propose(proposal_number, round_number, propose_value, &txns_binding);
                            
                            proposals.clear();
                            
                        } 
                    }
                    Message::Accept(proposal_number, leader_id, round_number, value) => {
                        println!("[Proposer] Received ACCEPT: {:?}", value);
                        accepted_values.push(value.clone());
                        // Count occurrences of each value in accepted_values
                        let mut value_counts = std::collections::HashMap::new();
                        for value in &accepted_values {
                            *value_counts.entry(value.clone()).or_insert(0) += 1;
                        }
                        
                        // Find the maximum count and its corresponding value
                        let (max_value, max_count) = value_counts.iter()
                            .max_by_key(|&(_, count)| count)
                            .map(|(value, count)| (value.clone(), *count))
                            .unwrap_or((String::new(), 0));
                        if max_count >= (NUM_ACCEPTORS / 2) + 1 {
                            println!("[Proposer] ACCEPT QUORUM REACHED");
                            println!("[Proposer] Sent ACCEPT to learner: {:?} accepted: {:?}", max_value, accepted_values);
                            accepted_values.clear();
                            for learner_tx in txns_binding.lock().unwrap().iter() {
                                learner_tx
                                    .send(Message::Accept(proposal_number, leader_id, round_number, max_value.clone()))
                                    .unwrap();
                            }
                            proposals.clear();
                        }
                    }
                    Message::RoundNumber(round_number) => {
                        println!("[Proposer] Received ROUND NUMBER: {:?}", round_number);
                        node.update_round_number(round_number, &txns_binding);
                    }
                    Message::Fail(value) => {
                        println!("[Proposer] Received FAIL {:?}", value);
                    }
                    Message::Terminate => {
                        println!("[Proposer] Received TERMINATE");
                        break;
                    }
                    // ACCEPTOR
                    Message::Prepare(proposal_number, leader_id, round_number, value) => {
                        node.handle_prepare(proposal_number, leader_id, round_number, value, &txns_binding);
                    }   
                    Message::Propose(proposal_number, leader_id, round_number, value) => {
                        node.handle_propose(proposal_number, leader_id, round_number, value, &txns_binding);
                    }
                    Message::Reset => {
                        println!("[Acceptor] Received RESET");
                        node.reset();
                    }
                    Message::Fail(value) => {
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
            }
        });
        nodes.push(handle);
    }
}
fn main() {
  
    let mut client = Client::new(0);
    let mut client2 = Client::new(1);
    let mut clients: Vec<Client> = vec![client.clone(), client2.clone()];
    let mut storage = Arc::new(Mutex::new(vec![]));
    // Nodes
    let mut nodes = vec![];
    // Channels
    let (txns, rxns) = setup_channels(NUM_NODES);
    let (client_txs, client_rxs) = setup_channels(NUM_CLIENTS);
   
    setup_nodes(&mut nodes, &mut storage, client_txs.clone(), txns.clone(), rxns.clone());
    client.consensus(None, "values".to_string(), txns.lock().unwrap()[0].clone());
    
    for client in clients.iter_mut() {
        client.await_stable_leader(client_rxs.clone());
    }
    
    thread::sleep(Duration::from_secs(1));
    client.send_to_stable_leader(None, "wabbit".to_string(), txns.lock().unwrap()[0].clone());
    client.send_to_stable_leader(None, "wabb2it".to_string(), txns.lock().unwrap()[0].clone());
    client.send_to_stable_leader(None, "wabitual".to_string(), txns.lock().unwrap()[0].clone());
    client2.send_to_stable_leader(Some(10), "wabitual".to_string(), txns.lock().unwrap()[0].clone());
    thread::sleep(Duration::from_secs(1));
    // client.consensus(Some(10), "wabbit".to_string(), proposer_txs[0].clone());
    client.send_to_stable_leader(Some(10), "∑avingwabbit".to_string(), txns.lock().unwrap()[0].clone());
    thread::sleep(Duration::from_secs(3));
    
    
    for txn in txns.lock().unwrap().iter() {
        txn.send(Message::Terminate).unwrap_or(println!("Failed to send TERMINATE message to node"));
    }

    for node in nodes {
        node.join().unwrap();
    }
    println!("storage: {:?}", storage.lock().unwrap());
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
        let mut storage = Arc::new(Mutex::new(vec![]));
        // Nodes
        let mut nodes = vec![];
        // Channels
        let (txns, rxns) = setup_channels(NUM_NODES);
        let (client_txs, client_rxs) = setup_channels(NUM_CLIENTS);
        setup_nodes(&mut nodes, &mut storage, client_txs.clone(), txns.clone(), rxns.clone());

        client.consensus(None, "values".to_string(), txns.lock().unwrap()[0].clone());
        thread::sleep(Duration::from_secs(2));
        for txn in txns.lock().unwrap().iter() {
            txn.send(Message::Terminate).unwrap();
        }
        for node in nodes {
            node.join().unwrap();
        }
        println!("storage: {:?}", storage.lock().unwrap());
        assert_eq!(storage.lock().unwrap().len(), 1);
    }
}
