use std::fmt;
#[derive(Debug, Clone)]
pub enum Message {
    Consensus(u64, String),
    Prepare(u64, String),
    Promise(u64, Option<u64>, String),
    Propose(u64, String),
    Accept(u64, String),
    Fail(String),
    Terminate,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            Message::Consensus(id, value) => format!("Consensus({id}, {value})"),
            Message::Prepare(id, value) => format!("Prepare({id}, {value})"),
            Message::Promise(id, accepted_value, value) => format!("Promise({}, {:?}, {})", id, accepted_value, value),
            Message::Propose(id, value) => format!("Propose({id}, {value})"),
            Message::Accept(id, value) => format!("Accept({id}, {value})"),
            Message::Fail(value) => format!("Fail({value})"),
            Message::Terminate => "Terminate".to_string(),
        };
        write!(f, "{msg}")
    }
}
