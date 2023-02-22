package raft

//
// this is an outline of the API that raft must expose to
// the service (or tester). see comments below for
// each of these functions for more details.
//
// rf = Make(...)
//   create a new Raft server.
// rf.Start(command interface{}) (index, term, isleader)
//   start agreement on a new log entry
// rf.GetState() (term, isLeader)
//   ask a Raft for its current term, and whether it thinks it is leader
// ApplyMsg
//   each time a new entry is committed to the log, each Raft peer
//   should send an ApplyMsg to the service (or tester)
//   in the same server.
//

import (
	//	"bytes"
	"log"
	"math/rand"
	"sync"
	"sync/atomic"
	"time"

	//	"6.5840/labgob"
	"6.5840/labrpc"
)

const ( 
	VotedNULL = -1
	MinElectionTimeOut = 150
	MaxElectionTimeOut = 300

)

type CurrentRole int

const (
	FOLLOWER CurrentRole = iota
	LEADER
	CANDIDATE
)



// as each Raft peer becomes aware that successive log entries are
// committed, the peer should send an ApplyMsg to the service (or
// tester) on the same server, via the applyCh passed to Make(). set
// CommandValid to true to indicate that the ApplyMsg contains a newly
// committed log entry.
//
// in part 2D you'll want to send other kinds of messages (e.g.,
// snapshots) on the applyCh, but set CommandValid to false for these
// other uses.
type ApplyMsg struct {
	CommandValid bool
	Command      interface{}
	CommandIndex int

	// For 2D:
	SnapshotValid bool
	Snapshot      []byte
	SnapshotTerm  int
	SnapshotIndex int
}


type LogEntry struct {
	
	// Node id 
	Me int 

	// Current Terms 
	Term int

	// Message Log 
	Command ApplyMsg
}

// A Go object implementing a single Raft peer.
type Raft struct {
	mu        sync.Mutex          // Lock to protect shared access to this peer's state
	peers     []*labrpc.ClientEnd // RPC end points of all peers
	persister *Persister          // Object to hold this peer's persisted state
	me        int                 // this peer's index into peers[]
	dead      int32               // set by Kill()

	// Your data here (2A, 2B, 2C).
	// Look at the paper's Figure 2 for a description of what
	// state a Raft server must maintain.
	// Persistent State of Each Node 
	State_ State

	// Volatile state in leaders
	LeaderState_ LeaderState

	// Volatile state in nodes 
	VolatileState_  VolatileState
}

// Volatile state on all Leaders 
type LeaderState struct { 
	// For each server, Index of the next Log entry to send to that servers
	// (initialised to leader last log index + 1)
	NextIndex []int 
	
	// For each server, index of highest log entry known to be replicated on server
	// (initialised to 0, increases monoticallu)
	MatchIndex []int
}

// Volatile State on all Servers
type VolatileState struct { 
	// Index of highest log entry known to be committed, initialised to 0  
	CommitIndex int 
	
	// Index of highest log entry applied to state machine 
	LastApplied int
}

// Persistent state on all Servers
type State struct { 
	// Latest term server has seen (initlaised to 0)
	CurrentTerm int 

	// Candidated Id that received vote in the current term
	VotedFor int 

	// Log entries 
	Log []LogEntry

	// Index of Candidate's last log entry 
	LastLogIndex int 

	// Term of Candidate's last log entry 
	LastLogTerm int 

	// Candidate's Role: Follower, Candidate, Leader
	CandidateRole CurrentRole

	// Election timer 
	// If time exceeds this value, Server starts new election
	ElectionTimer time.Time
}


// ----
// Invoked by leader to replicate log entries also used as hearts 
type AppendEntries struct { 
	// Leader's term 
	Term int 

	// Follower can redirect clients
	LeaderId int

	// Index of log entry immediately preceding new ones
	PrevLogIndex int 

	// Term of prevLogINdex entry
	PrevLogTerm int 

	// Log entries to store (empty for heartbeat; may send more than one for efficiencuy)
	entries []LogEntry

	// Leader's commmit Index
	LeaderCommit int 
}

type NodeReply struct { 
	// CurrentTerm, for the leader to update itself
	Term int 

	// True if follower contained entry matching prevLogIndex 
	// prevLogTerm 
	Success bool 
}


/*

*/
func (rf *Raft) sendAppendEntries(server int) bool {
	
	// Check if the target is alive else return false 
	// if isAlive := rf.peers[server].Call("Raft.Killed", isAlive); isAlive { return false }

	if rf.State_.CandidateRole != LEADER { return false }

	// Construct the AppendEntries RCP request to send over the AppendEntries 

	prevIndex :=  rf.LeaderState_.NextIndex[server]

	args := &AppendEntries{
		Term: rf.State_.CurrentTerm,
		LeaderId: rf.me,
		PrevLogIndex: prevIndex,
		PrevLogTerm: rf.State_.Log[prevIndex -1].Term,
		entries: rf.State_.Log[prevIndex :],
		LeaderCommit: rf.VolatileState_.CommitIndex,

	}
	var reply NodeReply



	if ok := rf.peers[server].Call("Raft.ProcessAppendEntries", &args, &reply); !ok { return false }

	
	// If success the leader should update the next index and match index for the target node 
	if reply.Term > rf.State_.CurrentTerm {
		rf.mu.Lock()

		rf.State_.CurrentTerm = reply.Term
		rf.State_.CandidateRole = FOLLOWER
		rf.State_.VotedFor = VotedNULL
		rf.mu.Unlock()
		return false
	}

	//	If success, update the targetNodes nextIndex and matchIndex
	if reply.Success {
		rf.mu.Lock()

		rf.LeaderState_.NextIndex[server] = args.PrevLogIndex + len(args.entries)
		rf.LeaderState_.MatchIndex[server] = args.PrevLogTerm + len(args.entries)

		// Ensure the log entry is only committd once it has been replicated on a majority of the nodes 
		for index := rf.VolatileState_.CommitIndex + 1; index < len(rf.State_.Log); index +=1 { 
			count := 1
			for j := range rf.peers { 
				if j == rf.me { continue }
				
				/*	
					Count the number of nodes that have replicated the log entry at that index
						MatchIndex = keeps track of the highest log entry that each node has replicated
				*/
				if rf.LeaderState_.MatchIndex[j] >= index { 

					if rf.getLastLogTerm() == rf.State_.CurrentTerm { 
						count +=1
					}

				}
			}

			/*
				If the majority of nodes have replicated the log entry at that index, 
				update the commit Index 
			*/
			if count > len(rf.peers) / 2 { 
				rf.mu.Lock()
				rf.VolatileState_.CommitIndex = index 
				rf.mu.Unlock()
			}
		}

		rf.mu.Unlock()

		return true

	// If not successfl, the leader
	// decrements the the nextIndex for the targetNote and retries the request with a smaller subset of 
	// the log entries 
	} else { 
		
		rf.mu.Lock()
		rf.LeaderState_.NextIndex[server] -=1
		rf.mu.Unlock()
		return false

	}
}


func (rf *Raft) ProcessAppendEntries(server int , args *AppendEntries, reply *NodeReply) {
	
	// Acquire the lock 
	rf.mu.Lock()

	defer rf.mu.Unlock()

	// Set Default 
	reply.Success = false

	// Check if the node is atleast update to date with the leader 
	if args.Term < rf.State_.CurrentTerm {
		rf.State_.CurrentTerm = args.Term
		return 
	}

	// REset the election timeout 
	rf.ResetElectionTime()

	// Update the current term and become a follower 
	if args.Term > rf.State_.CurrentTerm {
		rf.State_.CurrentTerm = args.Term
		rf.State_.CandidateRole = FOLLOWER
		return 
	}

	// Update the reply term 
	reply.Term = rf.State_.CurrentTerm

	// check if the previous log entry in the leaders request matches 
	if args.PrevLogIndex >= len(rf.State_.Log) || args.PrevLogTerm != rf.getLastLogTerm()  { return }

	// Append new entries to follower log
	rf.State_.Log = rf.State_.Log[: args.PrevLogIndex]
	rf.State_.Log = append(rf.State_.Log, args.entries...)

	if args.LeaderCommit > rf.VolatileState_.CommitIndex { 
		rf.VolatileState_.CommitIndex = Min(args.LeaderCommit, len(rf.State_.Log))
		// broadcast any waiting threads 
	}

	reply.Success = true
	
 
}


func (rf *State) GetTerm() int { 
	return rf.CurrentTerm
}

// return currentTerm and whether this server
// believes it is the leader.
func (rf *Raft) GetState() (int, bool) {

	rf.mu.Lock()

	// Defer Delay the execution until the nearby function returns  
	defer rf.mu.Unlock()

	return rf.State_.CurrentTerm, rf.State_.CandidateRole == LEADER
}

// save Raft's persistent state to stable storage,
// where it can later be retrieved after a crash and restart.
// see paper's Figure 2 for a description of what should be persistent.
// before you've implemented snapshots, you should pass nil as the
// second argument to persister.Save().
// after you've implemented snapshots, pass the current snapshot
// (or nil if there's not yet a snapshot).
func (rf *Raft) persist() {
	// Your code here (2C).
	// Example:
	// w := new(bytes.Buffer)
	// e := labgob.NewEncoder(w)
	// e.Encode(rf.xxx)
	// e.Encode(rf.yyy)
	// raftstate := w.Bytes()
	// rf.persister.Save(raftstate, nil)
}


// restore previously persisted state.
func (rf *Raft) readPersist(data []byte) {
	if data == nil || len(data) < 1 { // bootstrap without any state?
		return
	}
	// Your code here (2C).
	// Example:
	// r := bytes.NewBuffer(data)
	// d := labgob.NewDecoder(r)
	// var xxx
	// var yyy
	// if d.Decode(&xxx) != nil ||
	//    d.Decode(&yyy) != nil {
	//   error...
	// } else {
	//   rf.xxx = xxx
	//   rf.yyy = yyy
	// }
}


// the service says it has created a snapshot that has
// all info up to and including index. this means the
// service no longer needs the log through (and including)
// that index. Raft should now trim its log as much as possible.
func (rf *Raft) Snapshot(index int, snapshot []byte) {
	// Your code here (2D).


	return 
}


// example RequestVote RPC arguments structure.
// field names must start with capital letters!
//
//
// Invoked by candidates to gather votes
//
type RequestVoteArgs struct {
	
	// Candidates's term 
	Term int 
	
	// Candidate requesting vote
	CandidateId int 

	// Index of candidate's last log entry
	// LastLogIndex int 

	// Term of candidate's last log entry 
	// LastLogTerm int 
	// Your data here (2A, 2B).
}

// example RequestVote RPC reply structure.
// field names must start with capital letters!
type RequestVoteReply struct {
	// Your data here (2A).
	// For candidate to update itself
	Term int 

	// True means candidate received vote 
	VoteGranted bool 
}

// Set normal election timeout, with randomness
func SetElectionTime() time.Time{
	randomRange := rand.Intn(MaxElectionTimeOut - MinElectionTimeOut) 
	return time.Now().Add(time.Duration(randomRange) * time.Millisecond)
}


func (rf *Raft) ResetElectionTime() {
	timeoutDuration := time.Duration(rand.Intn(MaxElectionTimeOut-MinElectionTimeOut)+MinElectionTimeOut) * time.Millisecond
    rf.State_.ElectionTimer = time.Now().Add(timeoutDuration)
}
// Get the last log term 
func (rf *Raft) getLastLogTerm() int { 
	return rf.State_.Log[len(rf.State_.Log) -1 ].Term
}
// Get the Last Log index
func (rf *Raft) getLastLogIndex() int { 
	return len(rf.State_.Log) -1 
}


// 	example RequestVote RPC handler.
// 	Invoked by candidates to gather votes 
//	
// 	
//
//
//
//
//
//
//
//
//
func (rf *Raft) RequestVote(args *RequestVoteArgs, reply *RequestVoteReply) {
	// log.Println( rf)
	
	
	
	// Your code here (2A, 2B).
	
	rf.mu.Lock()
	defer rf.mu.Unlock()

	// IF currentNodes term is < Raft current term
	if args.Term < rf.State_.CurrentTerm { 
		reply.Term = rf.State_.CurrentTerm
		reply.VoteGranted = false
		return 
	}

	if args.Term > rf.State_.CurrentTerm {
		rf.State_.CandidateRole = FOLLOWER
		rf.State_.VotedFor = VotedNULL
		rf.State_.CurrentTerm = args.Term
		return 
	}

	// If Node voted for itself -- skip 
	// if rf.State_.VotedFor == args.CandidateId { return }

	// If the votedFor == nil or Canditate Id == nil and log is updated , grant vote 
	if rf.State_.VotedFor == VotedNULL || rf.State_.VotedFor == args.CandidateId {
		// Reply 
		reply.Term = rf.State_.CurrentTerm
		reply.VoteGranted = true
	
		// Update the server state 
		rf.ResetElectionTime()
		rf.State_.VotedFor = args.CandidateId
		rf.State_.CandidateRole = FOLLOWER
	
		return 
	}  

	// Any thign else 
	reply.Term = rf.State_.CurrentTerm
	reply.VoteGranted = false
}

// example code to send a RequestVote RPC to a server.
// server is the index of the target server in rf.peers[].
// expects RPC arguments in args.
// fills in *reply with RPC reply, so caller should
// pass &reply.
// the types of the args and reply passed to Call() must be
// the same as the types of the arguments declared in the
// handler function (including whether they are pointers).
//
// The labrpc package simulates a lossy network, in which servers
// may be unreachable, and in which requests and replies may be lost.
// Call() sends a request and waits for a reply. If a reply arrives
// within a timeout interval, Call() returns true; otherwise
// Call() returns false. Thus Call() may not return for a while.
// A false return can be caused by a dead server, a live server that
// can't be reached, a lost request, or a lost reply.
//
// Call() is guaranteed to return (perhaps after a delay) *except* if the
// handler function on the server side does not return.  Thus there
// is no need to implement your own timeouts around Call().
//
// look at the comments in ../labrpc/labrpc.go for more details.
//
// if you're having trouble getting RPC to work, check that you've
// capitalized all field names in structs passed over RPC, and
// that the caller passes the address of the reply struct with &, not
// the struct itself.
func (rf *Raft) sendRequestVote(server int, args *RequestVoteArgs, reply *RequestVoteReply) bool {
	
	ok := rf.peers[server].Call("Raft.RequestVote", args, reply)
	
	return ok
}


// the service using Raft (e.g. a k/v server) wants to start
// agreement on the next command to be appended to Raft's log. if this
// server isn't the leader, returns false. otherwise start the
// agreement and return immediately. there is no guarantee that this
// command will ever be committed to the Raft log, since the leader
// may fail or lose an election. even if the Raft instance has been killed,
// this function should return gracefully.
//
// the first return value is the index that the command will appear at
// if it's ever committed. the second return value is the current
// term. the third return value is true if this server believes it is
// the leader.
func (rf *Raft) Start(command interface{}) (int, int, bool) {
	index := -1
	term := -1
	isLeader := true

	// Your code here (2B).


	return index, term, isLeader
}

// the tester doesn't halt goroutines created by Raft after each test,
// but it does call the Kill() method. your code can use killed() to
// check whether Kill() has been called. the use of atomic avoids the
// need for a lock.
//
// the issue is that long-running goroutines use memory and may chew
// up CPU time, perhaps causing later tests to fail and generating
// confusing debug output. any goroutine with a long-running loop
// should call killed() to check whether it should stop.
func (rf *Raft) Kill() {
	atomic.StoreInt32(&rf.dead, 1)
	// Your code here, if desired.
}

func (rf *Raft) killed() bool {
	z := atomic.LoadInt32(&rf.dead)
	return z == 1
}



//  The ticker go routine starts a new election if this peer hasn't received heartbeats recently 
func (rf *Raft) ticker() {

	
	
	// Loop over the nodes while raft protocol is alive 
	for rf.killed() == false {
		var sleepDuration time.Duration 	

		log.Println("YES")
		// Your code here (2A)
		rf.mu.Lock()
		state := rf.State_.CandidateRole
		// Check if a leader election should be started.
		rf.mu.Unlock()

		switch state { 
			case FOLLOWER:
				log.Println("FOLLOWER")


				// now := time.Now()
				// timeLeft := now.Add(-10 * time.Second)
				timeOutDuration := time.Duration(rand.Intn(MaxElectionTimeOut - MinElectionTimeOut) *int(time.Millisecond) )
				
				timeOut := time.Now().Add(timeOutDuration)
				//	If a follower does not receive any communication from the leader within this timeframe
				//	it assumes that the leader has failed and starts a new election by trasitioning 
				//  to the Candidate State
				if rf.State_.ElectionTimer.After(timeOut) { 

					rf.mu.Lock()
					rf.State_.CandidateRole = CANDIDATE
					rf.mu.Unlock()
				}
			/*
				Once a follower becomes a candidate, 
				- increments the current term and send out RequestVote RPC
			*/
			case CANDIDATE:
				log.Println("CANDIDATE")

				rf.mu.Lock()

				rf.State_.CurrentTerm +=1
				rf.State_.VotedFor = rf.me
				rf.ResetElectionTime()				
				
				requestArgs := &RequestVoteArgs{
					Term: rf.State_.CurrentTerm,
					CandidateId: rf.me,
					// LastLogIndex: rf.getLastLogIndex(),
					// LastLogTerm: rf.getLastLogTerm(),
			
				}
				rf.mu.Unlock()
				// rf.persist()
				// Send AppendEntries RPCs to all other nodes 
				// if received AppendEntries RPCs from majority:
				// return continue
				for i := range rf.peers {
					if i != rf.me {
						var reply RequestVoteReply

						if rf.sendRequestVote(i, requestArgs, &reply) {
							log.Println("SENDING VOTES ")
							// Candidate collections votes for current term
							// grant := make(chan bool)
							go rf.CollectVotes(rf.State_.CurrentTerm, &reply)
						}
					}
				}

				time.Sleep(10 * time.Millisecond)

			
			case LEADER:
				rf.mu.Lock()
				
				log.Println("🚀 LEADER")

				// Leader resets its onw election timeout 
				rf.ResetElectionTime()

				time.Sleep(sleepDuration)

				// Set sleep duration 
				rf.mu.Unlock()
				
				sleepDuration = time.Until(rf.State_.ElectionTimer)
		}
		// pause for a random amount of time between 50 and 350
		// milliseconds.
		// timeDelay = 50 + (rand.Int63() % 300)
	}
}

// Collect Votes in the current term 
// If the majority votes has been received then the node to the Leader 
func (rf* Raft) CollectVotes(term int, reply *RequestVoteReply) { 
	var votes int32 = 1
	// done := false 

	if reply.Term > rf.State_.CurrentTerm { 
		rf.State_.CurrentTerm = reply.Term
		rf.State_.CandidateRole = FOLLOWER
		rf.State_.VotedFor = VotedNULL
		return 
	}

	if reply.VoteGranted { 
		atomic.AddInt32(&votes, 1)
	}

	// If count receives the majority of 
	if atomic.LoadInt32(&votes) > int32(len(rf.peers) / 2) { 
		// done = true
		rf.mu.Lock()
		rf.State_.CandidateRole = LEADER
		for nodes := range rf.peers { 
			if nodes != rf.me { 
				// go rf.sendAppendEntries(nodes)
			}
		}
		
		rf.mu.Unlock()
	}
}


/*
	After a node wins the election, it becomes the leader and starts sending periodic 
	AppendEntries RPCs to all the other nodes in the cluster to replicate its log 
	and keep them up to date with the latest state of the system

	The leader is responsible for coordinating the operations of the cluster and 
	ensuring that all nodes are in sync 

	When a client submits a request to the cluster, it sends the request to the leader, 
	- which appends the request to its own log and 
	- sends AppendEntries RPCs to the other nodes to replicate the log entry 

*/
func (rf *Raft) ReinitialisedAfterElection() {
	peerCount := len(rf.peers)
	
	lastLogIndex := rf.getLastLogIndex()
	// lastLogTerm := rf.State_.Log[len -1].Term

	rf.LeaderState_.NextIndex = make([]int, peerCount)

	for i := range rf.LeaderState_.NextIndex {
		rf.LeaderState_.NextIndex[i] = lastLogIndex + 1
	}


	rf.LeaderState_.MatchIndex = make([]int, peerCount)

	for i := range rf.LeaderState_.MatchIndex { 
		rf.LeaderState_.MatchIndex[i] = 0
	}

	rf.LeaderState_.MatchIndex[rf.me] = lastLogIndex 

	//	Send append Entries 

	// Sync 


}


// the service or tester wants to create a Raft server. the ports
// of all the Raft servers (including this one) are in peers[]. this
// server's port is peers[me]. all the servers' peers[] arrays
// have the same order. persister is a place for this server to
// save its persistent state, and also initially holds the most
// recent saved state, if any. applyCh is a channel on which the
// tester or service expects Raft to send ApplyMsg messages.
// Make() must return quickly, so it should start goroutines
// for any long-running work.
func Make(peers []*labrpc.ClientEnd, me int,
	persister *Persister, applyCh chan ApplyMsg) *Raft {
	rf := &Raft{}
	rf.peers = peers
	rf.persister = persister
	rf.me = me

	// Your initialization code here (2A, 2B, 2C).
	state := &State{
		CurrentTerm: 0,
		VotedFor: VotedNULL,
		Log: []LogEntry{},
		// LastLogIndex: 0,
		// LastLogTerm: 0,
		CandidateRole: FOLLOWER,
		ElectionTimer: SetElectionTime(),
	}
	volatilestate := &VolatileState{
		CommitIndex: 0,
		LastApplied: 0,
	}

	leaderState := &LeaderState{
		NextIndex: nil,
		MatchIndex: nil,
	}

	rf.State_ = *state
	rf.VolatileState_ = *volatilestate
	rf.LeaderState_ = *leaderState

	// initialize from state persisted before a crash
	rf.readPersist(persister.ReadRaftState())

	// start ticker goroutine to start elections
	go rf.ticker()



	return rf
}
