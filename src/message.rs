#[derive(Debug, Clone)]
pub enum Message {
    Consensus(u64, String),
    Prepare(u64, String),
    Promise(u64, Option<u64>, String),
    Propose(u64, String),
    Accept(u64, String),
    Fail,
}
