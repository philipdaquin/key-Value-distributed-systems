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
	// "log"
	// "log"
	// "fmt"
	// "fmt"
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
	MinElectionTimeOut = 800
	MaxElectionTimeOut = 1200
	// HeartbeatInterval = 100
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
	Command interface{}
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

	// Concurrent helpers
	channels Channels
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
	// ElectionTimer time.Time
}

// Concurrent Channels
type Channels struct { 
	winElection chan bool
	stepDown chan bool 
	grantVotes chan bool
	applyCh chan ApplyMsg
	heartBeatResp chan bool
}

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
	Entries []LogEntry

	// Leader's commmit Index
	LeaderCommit int 
}

type AppendEntryReply struct { 
	// CurrentTerm, for the leader to update itself
	Term int 

	// True if follower contained entry matching prevLogIndex 
	// prevLogTerm 
	Success bool 
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
	
	// Candidate requesting vote``
	CandidateId int 

	// Index of candidate's last log entry
	LastLogIndex int 

	// Term of candidate's last log entry 
	LastLogTerm int 
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


// =============================LEADER SENDING AppendEntries===============================
func (rf *Raft) ProcessAppendEntries(args *AppendEntries, reply *AppendEntryReply) {
	// Acquire the lock
	rf.mu.Lock()
	defer rf.mu.Unlock()

	// Check if received term is less than the current term, reject the request
	if args.Term < rf.State_.CurrentTerm {
		reply.Term = rf.State_.CurrentTerm
		reply.Success = false

		return
	}
	// Update the current term and become a follower if the received term is greater than the current term
	if args.Term > rf.State_.CurrentTerm {
		rf.stepDown(args.Term)
	}

	lastIndex := rf.getLastLogIndex()
	rf.sendMessage(rf.channels.heartBeatResp, true)

	// Update the reply term
	reply.Term = rf.State_.CurrentTerm
	// Set default reply to false
	reply.Success = false


	if args.PrevLogIndex > lastIndex { 

		return
	}


	// Check if the previous log entry in the leader's request matches with the follower's log
	if args.PrevLogIndex >= len(rf.State_.Log) || rf.State_.Log[args.PrevLogIndex].Term != args.PrevLogTerm {
		return
	}

	// Append new entries to follower log
	rf.State_.Log = rf.State_.Log[:args.PrevLogIndex+1]
	rf.State_.Log = append(rf.State_.Log, args.Entries...)
	reply.Success = true

	// Update commit index
	if args.LeaderCommit > rf.VolatileState_.CommitIndex {
		rf.VolatileState_.CommitIndex = Min(args.LeaderCommit, rf.getLastLogIndex())
		go rf.applyLogs()

	}
}

func (self *Raft) sendAppendEntries(server int, args *AppendEntries) {
	
	log.Println("💓 Sending appendEntries ...")

	reply :=&AppendEntryReply{}

	if ok := self.peers[server].Call("Raft.ProcessAppendEntries", args, &reply); !ok { 
		// log.Fatalln("❌❌ Failed to process AppendEntries")
		return  
	}


	self.mu.Lock()
	defer self.mu.Unlock()

	// If success the leader should update the next index and match index for the target node 
	if reply.Term > self.State_.CurrentTerm {
		// Update current term and become a follower 
		self.stepDown(self.State_.CurrentTerm)		
		return 
	} 
		
	//	If success, update the targetNodes nextIndex and matchIndex
	if reply.Success {
		
		self.LeaderState_.NextIndex[server] = args.PrevLogIndex + len(args.Entries) + 1
		// Ensure the match index is not out of range 
		matchIndex := args.PrevLogIndex + len(args.Entries)
		if matchIndex > self.LeaderState_.MatchIndex[server] { 
			self.LeaderState_.MatchIndex[server] = args.PrevLogIndex + len(args.Entries)
		}


		// todo!
		// Ensure the log entry is only committd once it has been replicated on a majority of the nodes 
		// for index := self.VolatileState_.CommitIndex + 1; index <= len(self.State_.Log); index +=1 { 
		// 	count := 1
		// 	for j := range self.peers { 
		// 		if j == self.me { continue }
		// 		/*	
		// 			Count the number of nodes that have replicated the log entry at that index
		// 				MatchIndex = keeps track of the highest log entry that each node has replicated
		// 		*/
		// 		if self.LeaderState_.MatchIndex[j] >= index && 
		// 			self.State_.Log[index - 1].Term == self.State_.CurrentTerm { 
		// 				count +=1
		// 		}
		// 	}

		// 	/*
		// 		If the majority of nodes have replicated the log entry at that index, 
		// 		update the commit Index 
		// 	*/
		// 	if count > len(self.peers) / 2 { 
		// 		self.VolatileState_.CommitIndex = index 

		// 		go self.applyLogs()
		// 		break 
		// 	}
		// }
		return 

	// If not successfl, the leader
	// decrements the the nextIndex for the targetNote and retries the request with a smaller subset of 
	// // the log entries 
	// } else { 
		
	// 	self.LeaderState_.NextIndex[server] -=1
		
	}
	return 
}

//
// When a node receives an AppendEntries message from the leader that includes
// information about the leader's commit index, the node updates it own commit index
// to match the leader's 
// 
// This means that the node has agreed to apply all log entries up to that commit index to its state machine
func (self *Raft) applyLogs() {
	self.mu.Lock()
	defer self.mu.Unlock()

	for i := self.VolatileState_.LastApplied + 1; i < self.VolatileState_.CommitIndex; i +=1 { 
		self.channels.applyCh <- ApplyMsg{
			CommandValid: true,
			Command: self.State_.Log[i].Command,
			CommandIndex: i,
		}
		self.VolatileState_.LastApplied = i
	}
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

	if rf.State_.CandidateRole == LEADER { 
		log.Println("✅✅ GOT A LEADER!! TERM: ", rf.State_.CurrentTerm)
	}

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




func (self *Raft) NewRequestVoteArgs() *RequestVoteArgs {
	return &RequestVoteArgs{
		Term: self.State_.CurrentTerm,
		CandidateId: self.me,
		LastLogIndex: self.getLastLogIndex(),
		LastLogTerm: self.getLastLogTerm(),
	}
}

// Set normal election timeout, with randomness
func (self *Raft) SetElectionTime() time.Duration {
	// timeoutDuration := 50 + (rand.Int63() % 300) * time.Hour.Milliseconds()
	timeoutDuration := time.Duration(360 + rand.Intn(240)) 
	// timeoutDuration := time.Duration(rand.Intn(MaxElectionTimeOut-MinElectionTimeOut)+MinElectionTimeOut) 
	// timeoutDuration := time.Duration(360 + rand.Intn(240))

	log.Println("🛫 New Election Timeout:", timeoutDuration)
	
	return time.Duration(timeoutDuration) 
}


// func (rf *Raft) ResetElectionTime() {
// 	timeoutDuration := time.Duration(rand.Intn(MaxElectionTimeOut-MinElectionTimeOut)+MinElectionTimeOut) * time.Millisecond
//     rf.State_.ElectionTimer = time.Now().Add(timeoutDuration)
// }
// Get the last log term 
func (rf *Raft) getLastLogTerm() int { 
	j := rf.State_.LastLogTerm
	if i := len(rf.State_.Log); i > 0 { 
		j =  rf.State_.Log[i - 1].Term
	}
	return j
}
// Get the Last Log index
func (rf *Raft) getLastLogIndex() int { 
	// j := rf.State_.LastLogIndex
	// if i := len(rf.State_.Log); i > 0 { 
	// 	j = rf.State_.Log[i - 1].Me
	// }
	// return j
	return len(rf.State_.Log) - 1
}

func (rf *Raft) IsLogUpdated(index , term int ) bool { 
	lastIndex, lastTerm := rf.getLastLogIndex(), rf.getLastLogTerm()
	if term == lastTerm { 
		return index > lastIndex
	}
	return term > lastTerm 
}
// Invoked by RPC call `sendRequestVote`
func (self *Raft) RequestVote(args *RequestVoteArgs, reply *RequestVoteReply) {
	
	// Your code here (2A, 2B).
	self.mu.Lock()
	defer self.mu.Unlock()
	// Persist 

	reply.Term = self.State_.CurrentTerm
	reply.VoteGranted = false

	// IF currentNodes term is < Raft current term
	// This means the current Candidate os outdated and there is another leader who
	// has been elected wiht a higer term
	if args.Term < self.State_.CurrentTerm { 
		reply.Term = self.State_.CurrentTerm
		reply.VoteGranted = false
		return 
	}

	// Step down to follower
	if reply.Term > self.State_.CurrentTerm { 
		self.stepDown(args.Term)
		return 
	}

	// If the votedFor == nil or Canditate Id == nil and log is updated 
	log.Println("👷 Validating a vote...")
	if (self.State_.VotedFor == VotedNULL || self.State_.VotedFor == args.CandidateId) && 
		self.IsLogUpdated(args.LastLogIndex, args.LastLogTerm) {
		// Reply 
		reply.Term = self.State_.CurrentTerm
		reply.VoteGranted = true
	
		// Update the server state 
		self.State_.VotedFor = args.CandidateId
		// self.State_.CandidateRole = FOLLOWER
		// self.ResetElectionTime()
		self.sendMessage(self.channels.grantVotes, true)
		return		
	}  
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
func (self *Raft) sendRequestVote(server int, args *RequestVoteArgs, reply *RequestVoteReply) {
	
	// Send a RequestVote RPC to peer wait for the reply
	if ok := self.peers[server].Call("Raft.RequestVote", args, reply); !ok { 
		// log.Fatalln("❌ RPC failed!")
		return 
	}

	self.mu.Lock()
	defer self.mu.Unlock()

	// Ensure the state is still CANDIDATE 
	if self.State_.CandidateRole != CANDIDATE || args.Term != self.State_.CurrentTerm || 
		reply.Term < self.State_.CurrentTerm { 
		return 
	}

	// Check if the reply's Term > Current Candidate's Term, meaning
	// ANother Candidate with a higher term has already been elected as the leader or
	// that the current leader has intiated a new election cycle with a higher term 
	//
	// Meaning, the current election cycle is outdated therefore, we need to 
	// reset the State to the NDOE to become a FOLLOWER again 
	if reply.Term > self.State_.CurrentTerm { 
		self.stepDown(args.Term)
		return 
	}

	// Collect Votes in the current term 
	// If the majority votes has been received then the node to the Leader 
	// func (rf* Raft) CollectVotes(term int, reply *RequestVoteReply) { 
	log.Println("👍 Collecting votes")
	var votes int32 = 1
	
	if reply.VoteGranted { 
		atomic.AddInt32(&votes, 1)
		// If count receives the majority of 
		if atomic.LoadInt32(&votes) == int32((len(self.peers) / 2) + 1  ) { 
			log.Println("🚀 LEADER", args.Term)
			// self.ReinitialisedAfterElection()


			// self.sendMessage(self.channels.winElection, true)
		}
	}
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
func (self *Raft) Start(command interface{}) (int, int, bool) {
	self.mu.Lock()
	defer self.mu.Unlock()
	// Your code here (2B).
	if self.State_.CandidateRole != LEADER { 
		return -1, self.State_.CurrentTerm, false
	}

	// idx := self.LeaderState_.NextIndex[self.me]
	term := self.State_.CurrentTerm
	self.State_.Log = append(self.State_.Log, LogEntry{
		Me: self.me,
		Term: term, 
		Command: command,
	})

	// self.LeaderState_.NextIndex[self.me] +=1
	// self.LeaderState_.MatchIndex[self.me] = idx


	return self.getLastLogIndex(), term, true
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
}
func (rf *Raft) killed() bool {
	z := atomic.LoadInt32(&rf.dead)
	return z == 1
}


func (self *Raft) stepDown(currentTerm int) {
	self.State_.CurrentTerm = currentTerm
	self.State_.CandidateRole = FOLLOWER
	self.State_.VotedFor = VotedNULL
	self.LeaderState_.NextIndex = nil
	self.LeaderState_.MatchIndex = nil
	if self.State_.CandidateRole != FOLLOWER { 
		self.sendMessage(self.channels.stepDown, true)
	}
}

func (self *Raft) sendMessage(message chan bool, value bool ) { 
	select { 
	case message <- value:
	default:
	}
}
//
// Reset channels 
//
func (self *Raft) resetChannels() {
	self.channels.stepDown = make(chan bool)
	self.channels.winElection = make(chan bool)
	self.channels.grantVotes = make(chan bool)
	self.channels.heartBeatResp = make(chan bool)
}

//	Change the current role of a node into : CANDIDATE, FOLLOWER
//
func (self *Raft) electSelf(role CurrentRole)  {
	self.mu.Lock()
	defer self.mu.Unlock()

	log.Println("💪 ELECTING NODE TO CANDIDATE")
	// Ensure the current role is not equal to the role desired  
	if self.State_.CandidateRole != role { return	}

	self.resetChannels()
	self.State_.CandidateRole = CANDIDATE
	self.State_.CurrentTerm +=1
	self.State_.VotedFor = self.me

	// Request all nodes for a Vote
	self.BroadCastRequestVote()
}


func (self *Raft) BroadCastRequestVote() { 
	log.Println("🚧🚧 BROADCASTING REQUEST VOTES")
	if self.State_.CandidateRole != CANDIDATE {return }

	// Initialise the request for votes 
	requestVote := self.NewRequestVoteArgs()
	var voteRequestReply RequestVoteReply

	for node := range self.peers { 
		if node == self.me { continue }
		go self.sendRequestVote(node, requestVote, &voteRequestReply) 
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
func (self *Raft) ReinitialisedAfterElection() {
	self.mu.Lock()
	defer self.mu.Unlock()
	
	if self.State_.CandidateRole != CANDIDATE { return }

	self.resetChannels()

	self.State_.CandidateRole = LEADER
	peerCount := len(self.peers)
	lastLogIndex := self.getLastLogIndex() + 1
	self.LeaderState_.NextIndex = make([]int, peerCount)
	self.LeaderState_.MatchIndex = make([]int, peerCount)
	

	for i := range self.peers { 
		self.LeaderState_.NextIndex[i] = lastLogIndex
	}
	
	// for i := range self.LeaderState_.NextIndex {
	// 	self.LeaderState_.NextIndex[i] = lastLogIndex
	// }

	// for i := range self.LeaderState_.MatchIndex { 
	// 	self.LeaderState_.MatchIndex[i] = 0
	// }
	// Set the match index for the leader to its own last log index 
	// self.LeaderState_.MatchIndex[self.me] = lastLogIndex 
	
	// Start sending heartbeat messages to all peers 
	self.SendHeartBeats()
}

func (self *Raft) SendHeartBeats() { 

	log.Println("💓💓 Sending heartbearts...")

	if self.State_.CandidateRole != LEADER { return }
	
	//	Send append Entries 
	for server := range self.peers { 
		if server != self.me {
			// Send an AppendEntries RPC with no entries to each peer
			

			// Check if the target is alive else return false
			log.Println("👷 Creating new AppendEntry")
			

			// Construct the AppendEntries RCP request to send over the AppendEntries
			prevIndex := self.LeaderState_.NextIndex[server] - 1
			prevTerm := self.State_.Log[prevIndex].Term
			entries := self.State_.Log[self.LeaderState_.NextIndex[server]: ]
			deepCopy := make([]LogEntry, len(entries))
			copy(deepCopy, entries)

			// Create the AppendEntries message
			args := AppendEntries{
				Term:         self.State_.CurrentTerm,
				LeaderId:     self.me,
				PrevLogIndex: prevIndex,
				PrevLogTerm:  prevTerm,
				Entries:      deepCopy,
				LeaderCommit: self.VolatileState_.CommitIndex,
			}


			go self.sendAppendEntries(server, &args)
		}

	}
}


//  The ticker go routine starts a new election if this peer hasn't received heartbeats recently 
func (self *Raft) ticker() {

	for !self.killed() {

		self.mu.Lock()
		
		currentState := self.State_.CandidateRole

		self.mu.Unlock()

		switch currentState { 
		case LEADER: 
			select {
				case <- self.channels.stepDown: 
				case <- time.After(120 * time.Millisecond):
					self.mu.Lock()
					self.SendHeartBeats()
					self.mu.Unlock()
			}
		case FOLLOWER:
			select {
				case <- self.channels.grantVotes:
				case <- self.channels.heartBeatResp:
				case <- time.After(self.SetElectionTime() * time.Millisecond):
					self.electSelf(FOLLOWER)
			}

		case CANDIDATE:
			select {
				case <- self.channels.stepDown:
				case <- self.channels.winElection:
					self.ReinitialisedAfterElection()

				// If the election timer has expired then start a new election 
				case <- time.After(self.SetElectionTime() * time.Millisecond):
					self.electSelf(CANDIDATE)
			}
		}
	}
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
//
func Make(peers []*labrpc.ClientEnd, me int, persister *Persister, applyCh chan ApplyMsg) *Raft {
	rf := &Raft{}
	rf.peers = peers
	rf.persister = persister
	rf.me = me

	// Your initialization code here (2A, 2B, 2C).
	state := &State{
		CurrentTerm: 0,
		VotedFor: VotedNULL,
		Log: []LogEntry{},
		LastLogIndex: 0,
		LastLogTerm: 0,
		CandidateRole: FOLLOWER,
		// ElectionTimer: time.Now().Add(SetElectionTime()) ,
	}

	// state.Log = append(state.Log, LogEntry{Term: 0})

	volatilestate := &VolatileState{
		CommitIndex: 0,
		LastApplied: 0,
	}

	leaderState := &LeaderState{
		NextIndex: nil,
		MatchIndex: nil,
	}

	channel := &Channels{
		winElection: make(chan bool),
		stepDown: make(chan bool),
		grantVotes: make(chan bool),
		applyCh: applyCh,
		heartBeatResp: make(chan bool),

	}

	rf.State_ = *state
	rf.VolatileState_ = *volatilestate
	rf.LeaderState_ = *leaderState
	rf.channels = *channel
	// initialize from state persisted before a crash
	rf.readPersist(persister.ReadRaftState())

	// start ticker goroutine to start elections
	go rf.ticker()



	return rf
}
