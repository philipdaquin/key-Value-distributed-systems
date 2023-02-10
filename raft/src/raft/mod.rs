pub mod message;
pub mod request;
pub mod client;
pub mod persister;
pub mod state;
pub mod node;


use futures::channel::mpsc::{UnboundedSender, Receiver};
use std::sync::Arc;

use message::ApplyMsg;
use anyhow::Result;
use client::RaftClient;

use self::{persister::Persister, state::State, request::{RequestVoteArgs, RequestVoteReply}};

trait RaftService { 
    fn new(
        peers: Vec<RaftClient>,
        candidate_id: usize, 
        persister: Box<dyn Persister>,
        apply_ch: UnboundedSender<ApplyMsg>
        
    ) -> Self;
    fn persist(&mut self);
    fn restore(&mut self, data: &[u8]);
    fn send_request_vote(&self, server: usize, arg: RequestVoteArgs) -> Receiver<Result<RequestVoteReply>>;
    fn start<M: labcodec::Message>(&self, command: &M) -> Result<(u64, u64)>;
    fn cond_install_snapshot(
        &mut self, 
        last_log_term: u64, 
        last_log_index: u64,
        snapshot: &[u8]
    ) -> bool;
    fn snapshot(&mut self, index: u64, snapshot: &[u8]);

}

pub struct Raft { 
    peers: Vec<RaftClient>,

    persister: Box<dyn Persister>,

    candidate_id: usize, 

    state: Arc<State>
}

impl RaftService for Raft {
    fn new(
        peers: Vec<RaftClient>,
        candidate_id: usize, 
        persister: Box<dyn Persister>,
        apply_ch: UnboundedSender<ApplyMsg>
        
    ) -> Self {
        todo!()
    }

    fn persist(&mut self) {
        todo!()
    }

    fn restore(&mut self, data: &[u8]) {
        todo!()
    }

    fn send_request_vote(&self, server: usize, arg: RequestVoteArgs) -> Receiver<Result<RequestVoteReply>> {
        todo!()
    }

    fn start<M: labcodec::Message>(&self, command: &M) -> Result<(u64, u64)> {
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
