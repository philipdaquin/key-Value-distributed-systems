
pub struct State { 
    pub term: u64,
    pub is_leader: bool
}


impl State { 
    pub fn term(&self) -> u64 {
        self.term
    }
    
    pub fn is_leade(&self) -> bool { 
        self.is_leader
    }
}