use super::Raft;
use anyhow::Result;

pub trait NodeObject { 
    fn new(raft: Raft) -> Self;
    fn start<M: labcodec::Message>(&self, command: &M) -> Result<(u64, u64)>;
    fn get_state(&self);
    fn term(&self);
    fn is_leader(&self);
    fn kill(&self);
    fn cond_install_snapshot(
        &mut self, 
        last_log_term: u64, 
        last_log_index: u64,
        snapshot: &[u8]
    ) -> bool;
    fn snapshot(&mut self, index: u64, snapshot: &[u8]);

}

#[derive(Clone)]
pub struct Node { 
    log: todo!(),
    current_term: u64,
    voted_for: u64
}

impl NodeObject for Node {
    fn new(raft: Raft) -> Self {
        todo!()
    }

    fn start<M: labcodec::Message>(&self, command: &M) -> Result<(u64, u64)> {
        todo!()
    }

    fn get_state(&self) {
        todo!()
    }

    fn term(&self) {
        todo!()
    }

    fn is_leader(&self) {
        todo!()
    }

    fn kill(&self) {
        todo!()
    }

    fn cond_install_snapshot(
        &mut self, 
        last_log_term: u64, 
        last_log_index: u64,
        snapshot: &[u8]
    ) -> bool {
        todo!()
    }

    fn snapshot(&mut self, index: u64, snapshot: &[u8]) {
        todo!()
    }
}