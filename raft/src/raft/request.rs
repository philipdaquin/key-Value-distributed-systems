


pub struct RequestVoteArgs { 
    term: u32, 
    candidate_id: u32, 
    last_log_index: u32, 
    last_log_term: u32
}

pub struct RequestVoteReply { 
    current_term: u32, 
    candidate_id: u32,
    last_log_index: u32, 
}