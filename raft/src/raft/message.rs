


/// As each Raft peer becomes aware that successive log entries are committed, 
/// the peer should an `ApplyMsg` to the service (or tester) on the same server
/// via the `apply_ch` passed to the `Raft::new`
pub enum ApplyMsg { 
    Command { 
        data: Vec<u8>,
        index: u64
    },
    SnapShot { 
        data: Vec<u8>,
        term: u64, 
        index: u64
    }
}