use std::fmt;
#[derive(Debug, Clone)]
pub enum Message {
    Consensus(u64, String),
    StableConsensus(u64, String),
    Prepare(u64, u64, String),
    Promise(u64, u64, Option<u64>, String),
    Propose(u64, u64, String),
    Accept(u64, u64, String),
    RoundNumber(u64),
    Reset,
    Fail(String),
    Terminate,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            Message::Consensus(id, value) => format!("Consensus({id}, {value})"),
            Message::StableConsensus(id, value) => format!("StableConsensus({id}, {value})"),
            Message::Prepare(id, round_number, value) => format!("Prepare({}, {}, {})", id, round_number, value),
            Message::Promise(id, round_number, accepted_value, value) => format!("Promise({}, {}, {:?}, {})", id, round_number, accepted_value, value),
            Message::Propose(id, round_number, value) => format!("Propose({}, {}, {})", id, round_number, value),
            Message::Accept(id, round_number, value) => format!("Accept({}, {}, {})", id, round_number, value),
            Message::RoundNumber(round_number) => format!("RoundNumber({round_number})"),
            Message::Reset => "Reset".to_string(),
            Message::Fail(value) => format!("Fail({value})"),
            Message::Terminate => "Terminate".to_string(),
        };
        write!(f, "{msg}")
    }
}
