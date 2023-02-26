use std::sync::Arc;

use futures::channel::mpsc::{UnboundedSender, Receiver};
use tracing::subscriber::SetGlobalDefaultError;
use persister::Persister;
use errors::Result;
use crate::proto::raftpb::{RequestVoteArgs, RequestVoteReply, RaftClient, RaftService};

pub mod config;
pub mod errors;
pub mod persister;

use self::errors::*;
use self::persister::*;
use crate::proto::raftpb::*;

/// as each Raft peer becomes aware that successive log entries are
/// committed, the peer should send an ApplyMsg to the service (or
/// tester) on the same server, via the applyCh passed to Make(). set
/// CommandValid to true to indicate that the ApplyMsg contains a newly
/// committed log entry.
///
/// in part 2D you'll want to send other kinds of messages (e.g.,
/// snapshots) on the applyCh, but set CommandValid to false for these
/// other uses.
pub enum ApplyMsg { 
    Command {
        data: Vec<u8>,
        index: u64
    },
    Snapshot {
        data: Vec<u8>, 
        term: u64,
        index: u64
    }
}


pub struct Raft { 

    // RPC end points of all peers
    peers: Vec<RaftClient>,
    // Object to hold this peer's persisted state
    persister: Box<dyn Persister>,
    // this peer's index into peers[]
    me: usize, 

    pub state: Arc<State>
    
    // Your data here (2A, 2B, 2C).
	// Look at the paper's Figure 2 for a description of what
	// state a Raft server must maintain.
}

// The state of the RAFT peers
#[derive(Clone, Debug, Default)]
pub struct State { 
    pub term: u64, 
    pub is_leader: bool
}

impl State { 
    /// Return currentTerm and whether this server believes 
    /// it is the leader 
    fn get_state(&self) -> Self { 
        Self { 
            term: self.term, 
            is_leader: self.is_leader
        }
    } 

    fn term(&self) -> u64 { 
        self.term
    }

    fn is_leader(&self) -> bool { 
        self.is_leader
    }

}

impl Raft { 
    /// the service or tester wants to create a Raft server. the ports
    /// of all the Raft servers (including this one) are in peers. this
    /// server's port is peers[me]. all the servers' peers arrays
    /// have the same order. persister is a place for this server to
    /// save its persistent state, and also initially holds the most
    /// recent saved state, if any. apply_ch is a channel on which the
    /// tester or service expects Raft to send ApplyMsg messages.
    /// This method must return quickly.
    pub fn new(
        peers: Vec<RaftClient>,
        me: usize, 
        persister: Box<dyn Persister>,
        apply_ch: UnboundedSender<ApplyMsg>
    ) -> Self { 
        todo!()
    }

    /// save Raft's persistent state to stable storage,
    /// where it can later be retrieved after a crash and restart.
    /// see paper's Figure 2 for a description of what should be persistent.
    /// before you've implemented snapshots, you should pass nil as the
    /// second argument to persister.Save().
    /// after you've implemented snapshots, pass the current snapshot
    /// (or nil if there's not yet a snapshot).
    fn persist(&mut self) {
        todo!()
    }


    /// Restore previously persisted state
    fn read_persist(&mut self, data: &[u8]) {
        if data.is_empty() { 
            todo!()
        }

        todo!()
    }

    /// example code to send a RequestVote RPC to a server.
    /// server is the index of the target server in rf.peers[].
    /// expects RPC arguments in args.
    /// fills in *reply with RPC reply, so caller should
    /// pass &reply.
    /// the types of the args and reply passed to Call() must be
    /// the same as the types of the arguments declared in the
    /// handler function (including whether they are pointers).
    ///
    /// The labrpc package simulates a lossy network, in which servers
    /// may be unreachable, and in which requests and replies may be lost.
    /// Call() sends a request and waits for a reply. If a reply arrives
    /// within a timeout interval, Call() returns true; otherwise
    /// Call() returns false. Thus Call() may not return for a while.
    /// A false return can be caused by a dead server, a live server that
    /// can't be reached, a lost request, or a lost reply.
    ///
    /// Call() is guaranteed to return (perhaps after a delay) *except* if the
    /// handler function on the server side does not return.  Thus there
    /// is no need to implement your own timeouts around Call().
    ///
    /// look at the comments in ../labrpc/labrpc.go for more details.
    ///
    /// if you're having trouble getting RPC to work, check that you've
    /// capitalized all field names in structs passed over RPC, and
    /// that the caller passes the address of the reply struct with &, not
    /// the struct itself.
    fn send_request_vote(&self, server: usize, args: RequestVoteArgs) -> Receiver<Result<RequestVoteReply>> {
        todo!()
    }  

    
    /// the service using Raft (e.g. a k/v server) wants to start
    /// agreement on the next command to be appended to Raft's log. if this
    /// server isn't the leader, returns false. otherwise start the
    /// agreement and return immediately. there is no guarantee that this
    /// command will ever be committed to the Raft log, since the leader
    /// may fail or lose an election. even if the Raft instance has been killed,
    /// this function should return gracefully.
    ///
    /// the first return value is the index that the command will appear at
    /// if it's ever committed. the second return value is the current
    /// term. the third return value is true if this server believes it is
    /// the leader.
    fn start<M: labcodec::Message>(&self, command: &M) -> Result<(u64, u64)> {
        todo!()
    }

    /// the service says it has created a snapshot that has
    /// all info up to and including index. this means the
    /// service no longer needs the log through (and including)
    /// that index. Raft should now trim its log as much as possible.
    fn cond_install_snapshot(&mut self, included_term: u64, last_index: u64, snapshot: &[u8]) -> bool { 
        
        // 2D
        
        todo!()
    }

}

// Choose concurrency paradigm.
//
// You can either drive the raft state machine by the rpc framework,
//
// ```rust
// struct Node { raft: Arc<Mutex<Raft>> }
// ```
//
// or spawn a new thread runs the raft state machine and communicate via
// a channel.
//
// ```rust
// struct Node { sender: Sender<Msg> }
// ```
#[derive(Clone)]
pub struct Node {
    // Your code here.
}

impl Node { 

    /// Create a new Raft service  
    pub fn new(raft: Raft) -> Self { 
        todo!()
    }


    pub fn term(&self) -> u64 {
        todo!()
    }
    pub fn is_leader(&self) -> bool {
        todo!()
    }
    pub fn get_state(&self) -> State {
        todo!()
    }

    pub fn send_request_vote(&self, server: usize, args: RequestVoteArgs) -> Receiver<Result<RequestVoteReply>> {
        todo!()
    }  

    pub fn kill(&self) {
        todo!()
    }

    pub fn start<M: labcodec::Message>(&self, command: &M) -> Result<(u64, u64)> {
        todo!()
    }

    pub fn cond_install_snapshot(&self, included_term: u64, last_index: u64, snapshot: &[u8]) -> bool { 
        // 2D
        todo!()
    }
    
    /// the service says it has created a snapshot that has
    /// all info up to and including index. this means the
    /// service no longer needs the log through (and including)
    /// that index. Raft should now trim its log as much as possible.
    fn snapshot(&self, index: u64, snapshot: &[u8]) {
        todo!()
    }
}

#[async_trait::async_trait]
impl RaftService for Node {
    // example RequestVote RPC handler.
    //
    // CAVEATS: Please avoid locking or sleeping here, it may jam the network.
    async fn request_vote(&self, args: RequestVoteArgs) -> labrpc::Result<RequestVoteReply> {
        // Your code here (2A, 2B).
        todo!()
    }
}