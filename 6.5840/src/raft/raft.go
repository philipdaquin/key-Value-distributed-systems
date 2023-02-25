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
	"bytes"
	"log"
	"math/rand"
	"sync"
	"sync/atomic"
	"time"
	"6.5840/labgob"
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

type SnapshotCmd struct {
	index int 
	snapshot []byte
}

// Log entry 
type LogEntry struct {
	
	// // Node id 
	Me int 

	// Current Terms 
	Term int

	// Message Log 
	Command interface{}
}
// func (rf *Raft) ResetElectionTime() {
// 	timeoutDuration := time.Duration(rand.Intn(MaxElectionTimeOut-MinElectionTimeOut)+MinElectionTimeOut) * time.Millisecond
//     rf.State_.ElectionTimer = time.Now().Add(timeoutDuration)
// }
// Get the last log term 
func (rf *Raft) getLastLogTerm() int { 
	return rf.State_.Log[rf.getLastLogIndex()].Term
}
// Get the Last Log index
func (rf *Raft) getLastLogIndex() int { 
	return len(rf.State_.Log) - 1
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
	VoteCount int 
}

// Concurrent Channels
type Channels struct { 
	winElection chan bool
	stepDown chan bool 
	grantVotes chan bool
	applyCh chan ApplyMsg
	heartBeatResp chan bool
	snapshotCh chan SnapshotCmd
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

	ConflictIndex int 
	ConflictTerm int
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

// Set normal election timeout, with randomness
func (self *Raft) setElectionTime() time.Duration {
	log.Println("🛫 New Election Timeout:")

	return time.Duration(360 + rand.Intn(240)) 
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

func (rf *Raft) IsLogUpdated(cindex , cterm int ) bool { 
	log.Println("🤔 Checking if the log is updated....")
	lastIndex, lastTerm := rf.getLastLogIndex(), rf.getLastLogTerm()
	if cterm == lastTerm { 
		return cindex >= lastIndex
	}
	return cterm > lastTerm 
}

//
// When a node receives an AppendEntries message from the leader that includes
// information about the leader's commit index, the node updates it own commit index
// to match the leader's 
// 
// This means that the node has agreed to apply all log entrides up to that commit index to its state machine
func (self *Raft) applyLogs() {
	self.mu.Lock()
	defer self.mu.Unlock()

	for i := self.VolatileState_.LastApplied + 1; i <= self.VolatileState_.CommitIndex; i +=1 { 
		self.channels.applyCh <- ApplyMsg{
			CommandValid: true,
			Command: self.State_.Log[i].Command,
			CommandIndex: i,
		}
		self.VolatileState_.LastApplied = i
	}
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

func (self *Raft) stepDown(currentTerm int) {

	currentRole := self.State_.CandidateRole

	self.State_.CurrentTerm = currentTerm
	self.State_.CandidateRole = FOLLOWER
	self.State_.VotedFor = VotedNULL
	// self.LeaderState_.NextIndex = nil
	// self.LeaderState_.MatchIndex = nil
	if currentRole != FOLLOWER { 
		self.sendMessage(self.channels.stepDown, true)
	}
}


// =============================LEADER SENDING AppendEntries==================================
func (rf *Raft) ProcessAppendEntries(args *AppendEntries, reply *AppendEntryReply) {

	log.Println("🧑‍🏭 Processing AppenEntries")

	// Acquire the lock
	rf.mu.Lock()
	defer rf.mu.Unlock()
	defer rf.persist()

	// Check if received term is less than the current term, reject the request
	log.Println("1. Check if there's new leader ")
	if args.Term < rf.State_.CurrentTerm {
		reply.Term = rf.State_.CurrentTerm
		reply.Success = false
		reply.ConflictIndex = -1
		reply.ConflictTerm = -1

		return
	}
	// Update the current term and become a follower if the received term is greater than the current term
	log.Println("2. Check if there's new leader ")
	if args.Term > rf.State_.CurrentTerm {
		rf.stepDown(args.Term)
	}

	lastIndex := rf.getLastLogIndex()
	rf.sendMessage(rf.channels.heartBeatResp, true)

	// Update the reply term
	reply.Term = rf.State_.CurrentTerm
	reply.Success = false
	reply.ConflictIndex = -1
	reply.ConflictTerm = -1

	log.Println("Check if the previous index > last index ")
	if args.PrevLogIndex > lastIndex { 
		reply.ConflictIndex = lastIndex + 1
		return
	}

	// Log consistency fails 
	log.Println("Checking Log consistency")
	if xterm := rf.State_.Log[args.PrevLogIndex].Term; xterm != args.PrevLogTerm { 
		reply.ConflictTerm = xterm 

		for i:= args.PrevLogIndex; i >= 0 && rf.State_.Log[i].Term == xterm; i -=1 {
			reply.ConflictIndex = i 
		}
		reply.Success = true
		return 
	}

	prevIdx := args.PrevLogIndex + 1
	entryIdx := 0

	for prevIdx < lastIndex + 1 && entryIdx < len(args.Entries) {
		if rf.State_.Log[prevIdx].Term != args.Entries[entryIdx].Term { 
			break;
		}
		prevIdx +=1
		entryIdx +=1
	}
	// Append new entries to follower log
	rf.State_.Log = rf.State_.Log[:prevIdx]
	args.Entries = args.Entries[entryIdx: ]
	rf.State_.Log = append(rf.State_.Log, args.Entries...)
	reply.Success = true

	// Update commit index
	if args.LeaderCommit > rf.VolatileState_.CommitIndex {

		lastIndex_ := rf.getLastLogIndex()

		rf.VolatileState_.CommitIndex = Min(args.LeaderCommit, lastIndex_)
		go rf.applyLogs()
	}
}

func (self *Raft) sendAppendEntries(server int, args *AppendEntries, reply *AppendEntryReply) {
	
	log.Println("🛳️ Sending appendEntries ...")


	ok := self.peers[server].Call("Raft.ProcessAppendEntries", args, reply); 
	
	if !ok { 
		// log.Fatalln("❌❌ Failed to process AppendEntries")
		return  
	}


	self.mu.Lock()
	defer self.mu.Unlock()
	// Persist 
	defer self.persist()


	if self.State_.CandidateRole != LEADER || 
		args.Term != self.State_.CurrentTerm || 
		reply.Term < self.State_.CurrentTerm { 
		
			return 
	}

	// If success the leader should update the next index and match index for the target node 
	if reply.Term > self.State_.CurrentTerm {
		// Update current term and become a follower 
		self.stepDown(args.Term)		
		return 
	} 
		
	//	If success, update the targetNodes nextIndex and matchIndex
	if reply.Success {
		matchIndex := args.PrevLogIndex + len(args.Entries)
		
		// Ensure the match index is not out of range 
		if matchIndex > self.LeaderState_.MatchIndex[server] { 
			self.LeaderState_.MatchIndex[server] = matchIndex
		}
		self.LeaderState_.NextIndex[server] = self.LeaderState_.MatchIndex[server] + 1

	} else if reply.ConflictTerm < 0 {
		// Follower's log is shorter than the leader's
		self.LeaderState_.NextIndex[server] = reply.ConflictIndex
		self.LeaderState_.MatchIndex[server] = self.LeaderState_.NextIndex[server] - 1

	} else { 
		
		nextIndex := self.getLastLogIndex()
		for; nextIndex >= 0; nextIndex -- {
			if self.State_.Log[nextIndex].Term == reply.ConflictTerm { break }
		}

		if nextIndex < 0 { 
			self.LeaderState_.NextIndex[server] = reply.ConflictIndex
		} else { 
			self.LeaderState_.NextIndex[server] = nextIndex
		}
		
		self.LeaderState_.MatchIndex[server] = self.LeaderState_.NextIndex[server] - 1
	}


	// Ensure the log entry is only committd once it has been replicated on a majority of the nodes 
	for n := self.getLastLogIndex(); n >= self.VolatileState_.CommitIndex; n-- {
		count := 1
		if self.State_.Log[n].Term == self.State_.CurrentTerm {
			for i := 0; i < len(self.peers); i++ {
				if i != self.me && self.LeaderState_.MatchIndex[i] >= n {
					count++
				}
			}
		}
		if count > len(self.peers) / 2 {
			self.VolatileState_.CommitIndex = n
			go self.applyLogs()
			break
		}
	}
}
// ===========================================================================================


// ============================= PERSISTENCE =================================================
// save Raft's persistent state to stable storage,
// where it can later be retrieved after a crash and restart.
// see paper's Figure 2 for a description of what should be persistent.
// before you've implemented snapshots, you should pass nil as the
// second argument to persister.Save().
// after you've implemented snapshots, pass the current snapshot
// (or nil if there's not yet a snapshot).
func (self *Raft) persist() {
	buffer := new(bytes.Buffer)
	encoder := labgob.NewEncoder(buffer)

	if encoder.Encode(self.State_.CurrentTerm) != nil || 
		encoder.Encode(self.State_.VotedFor) != nil || 
		encoder.Encode(self.State_.Log) != nil { panic("Failed to encode RAFT persistent state ") 
	}
	data := buffer.Bytes()

	self.persister.SaveRaftState(data)
}
// restore previously persisted state.
func (rf *Raft) readPersist(data []byte) {
	if data == nil || len(data) < 1 { // bootstrap without any state?
		return
	}

	readerBuffer := bytes.NewBuffer(data)
	deserialised := labgob.NewDecoder(readerBuffer)

	if deserialised.Decode(&rf.State_.CurrentTerm) != nil || 
		deserialised.Decode(&rf.State_.VotedFor) != nil || 
		deserialised.Decode(&rf.State_.Log) != nil { 
			panic("Failed to decode Raft Persistent State")
		}
}

func (self *Raft) getRaftState() []byte { 
	writerBuffer := new(bytes.Buffer)
	encoded := labgob.NewEncoder(writerBuffer)
	if  encoded.Encode(self.State_.CurrentTerm) != nil || 
		encoded.Encode(self.State_.VotedFor) != nil || 
		encoded.Encode(self.State_.Log) != nil || 
		encoded.Encode(self.State_.LastLogIndex) != nil || 
		encoded.Encode(self.State_.LastLogTerm) != nil { 
			return nil
		}
	return writerBuffer.Bytes()
}


// ================================Log Compaction and Snapshotting ===========================
//	Invoked by leader to send chunks of a snapshot to a follower 
//	Leaders send the chunk in order 
type InstallSnapShotArgs struct {
	
	// Leader's Term  
	Term int 

	// So follower can redirect clients
	LeaderId int 

	// The snapshot replaces all entries up through and including this index 
	//
	// Once a entries up through the last included index, as well as any prior snapshot
	LastIncludedIndex int 

	// Term of this entry 
	LastIncludedTerm int 

	// Raw bytes of the snapshot chunk, starting at offset 
	Data []byte

	// True if this is the last chunk 
	Done bool
}

type InstallSnapshotReply struct { 
	// Current Term for the 
	Term int 
}


//  The conditional version of the `InstallSnapshot` function that checks if the snapshot is more 
//  recent than the state machine's current snapshot. 
//
//	Return True -- If the snapshot is more recent, and apply the snapshot to the statemachine
//	
//  Otherwise, It does nothing and return False
//		It means the snapshot is an older version
//		This is because RAFT, and before CondInstall was invoked by the server
// 		It is not OK for RAFT to go back to snapshot, CondInstallSnapshot should just return False so that the server knows it 
// 		it shouldnt switch to the snapshot 
//
func (self *Raft) CondInstallSnapshot(lastIncludedTerm int, lastIncludedIndex int, snapshot []byte) bool {
	self.mu.Lock()
	defer self.mu.Unlock()

	if lastIncludedIndex <= self.VolatileState_.CommitIndex { return false }

	if lastIncludedIndex >= self.getLastLogIndex() {
		self.State_.Log = make([]LogEntry, 1)
	} else { 
		self.State_.Log = self.State_.Log[lastIncludedIndex - self.State_.Log[0].Me: ]
	}
	self.State_.Log[0].Me = lastIncludedIndex
	self.State_.Log[0].Term = lastIncludedTerm

	self.VolatileState_.LastApplied = lastIncludedIndex
	self.VolatileState_.CommitIndex = lastIncludedIndex

	self.persister.SaveStateAndSnapshot(self.getRaftState(), snapshot)

	
	return true
}

func (self *Raft) InstallSnapshot(args *InstallSnapShotArgs, reply *InstallSnapshotReply) {
	self.mu.Lock()
	defer self.mu.Unlock()


	reply.Term = self.State_.CurrentTerm

	if args.Term < self.State_.CurrentTerm { return }

	if args.Term > self.State_.CurrentTerm { 
		self.stepDown(args.Term)

		self.persist()
	}

	if args.LastIncludedIndex <= self.VolatileState_.CommitIndex { return }

	go func()  {
		self.channels.applyCh <- ApplyMsg{
			SnapshotValid: true,
			Snapshot: args.Data,
			SnapshotTerm: args.LastIncludedTerm,
			SnapshotIndex: args.LastIncludedIndex,
		}
	}()

}

// Send an IntallSnapshot RPC to a node 
func (self *Raft) sendInstallSnapshot(server int, args *InstallSnapShotArgs, reply *InstallSnapshotReply) bool {
	ok := self.peers[server].Call("Raft.InstallSnapshot", args, reply); 
	return ok
}

// 	At a high level, the Snapshot mehtod is responsible for creating a snapshot 
// 	of the current state of the Raft System, including the log entries and any associated 
// 	state machine data.
//
//	Once created, we can send it to the followers as part of the normal log replication process
// 	This involves sending a special message to the follower that includes the snapshot data, along with 
// 	with information about the last log entry that the follower has already recevied 
//
func (self *Raft) Snapshot(index int, snapshot []byte) {
	log.Println("💽 Snapshot ")
	// self.channels.snapshotCh <- SnapshotCmd{
	// 	index: index, 
	// 	snapshot: snapshot,
	// }
	self.mu.Lock()
	defer self.mu.Unlock()

	prevSnapIndex := self.State_.Log[0].Me
	if index <= prevSnapIndex { return }

	self.State_.Log = self.State_.Log[index - prevSnapIndex:]
	self.State_.Log[0].Command = nil

	self.persister.SaveStateAndSnapshot(self.getRaftState(), snapshot)

}





// ================================== Broadcasting Request Votes =============================

// Invoked by RPC call `sendRequestVote`
func (self *Raft) RequestVote(args *RequestVoteArgs, reply *RequestVoteReply) {
	
	// Your code here (2A, 2B).
	self.mu.Lock()
	defer self.mu.Unlock()
	defer self.persist()

	// IF currentNodes term is < Raft current term
	// This means the current Candidate os outdated and there is another leader who
	// has been elected wiht a higher term
	if args.Term < self.State_.CurrentTerm { 
		reply.Term = self.State_.CurrentTerm
		reply.VoteGranted = false
		return 
	}

	// Step down to follower
	if args.Term > self.State_.CurrentTerm { 
		self.stepDown(args.Term)
	}

	reply.Term = self.State_.CurrentTerm
	reply.VoteGranted = false
	
	// If the votedFor == nil or Canditate Id == nil and log is updated 
	log.Println("👷 Validating a vote...")
	if (self.State_.VotedFor == VotedNULL || 
		self.State_.VotedFor == args.CandidateId) && 
		self.IsLogUpdated(args.LastLogIndex, args.LastLogTerm) {
		
		reply.Term = self.State_.CurrentTerm
		reply.VoteGranted = true
		self.State_.VotedFor = args.CandidateId
		self.sendMessage(self.channels.grantVotes, true)
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
	ok := self.peers[server].Call("Raft.RequestVote", args, reply)
	
	if !ok { 
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
		self.persist()
		return 
	}

	// Collect Votes in the current term 
	// If the majority votes has been received then the node to the Leader 
	if reply.VoteGranted { 
		self.State_.VoteCount +=1
		// If count receives the majority of 
		if self.State_.VoteCount == len(self.peers) / 2 + 1 { 
			log.Println("🚀 LEADER", args.Term)
			self.sendMessage(self.channels.winElection, true)
		}
	}
}

func (self *Raft) BroadCastRequestVote() { 
	log.Println("🚧🚧 BROADCASTING REQUEST VOTES")
	if self.State_.CandidateRole != CANDIDATE {return }

	// Initialise the request for votes 
	requestVote := RequestVoteArgs{
		Term: self.State_.CurrentTerm,
		CandidateId: self.me,
		LastLogIndex: self.getLastLogIndex(),
		LastLogTerm: self.getLastLogTerm(),
	}

	for node := range self.peers { 
		if node != self.me { 
			go self.sendRequestVote(node, &requestVote, &RequestVoteReply{}) 
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

	term := self.State_.CurrentTerm
	self.State_.Log = append(self.State_.Log, LogEntry{
		Me: self.getLastLogIndex(),
		Term: term, 
		Command: command,
	})

	self.persist()


	return self.getLastLogIndex(), term, true
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
	self.State_.VoteCount = 1
	self.persist()

	// Request all nodes for a Vote
	self.BroadCastRequestVote()
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

	log.Println("😎 CONVERTING TO LEADER")
	self.mu.Lock()
	defer self.mu.Unlock()
	
	if self.State_.CandidateRole != CANDIDATE { return }

	self.resetChannels()

	self.State_.CandidateRole = LEADER
	self.LeaderState_.NextIndex = make([]int, len(self.peers))

	self.LeaderState_.MatchIndex = make([]int, len(self.peers))
	
	lastLogIndex := self.getLastLogIndex() + 1

	for i := range self.peers { 
		// log.Println(self.peers[i])
		self.LeaderState_.NextIndex[i] = lastLogIndex
	}
	
	// for i := range self.LeaderState_.MatchIndex { 
	// 	self.LeaderState_.MatchIndex[i] = 0
	// }
	// // Set the match index for the leader to its own last log index 
	// self.LeaderState_.MatchIndex[self.me] = lastLogIndex 
	// // Start sending heartbeat messages to all peers

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

			args := AppendEntries{}
			
			args.Term = self.State_.CurrentTerm
			args.LeaderId = self.me
			args.PrevLogIndex = self.LeaderState_.NextIndex[server] - 1
			args.PrevLogTerm = self.State_.Log[args.PrevLogIndex].Term
			args.LeaderCommit = self.VolatileState_.CommitIndex
			entries := self.State_.Log[self.LeaderState_.NextIndex[server]: ]

			args.Entries = make([]LogEntry, len(entries))

			copy(args.Entries, entries)


			var reply AppendEntryReply

			// Create the AppendEntries message
			go self.sendAppendEntries(server, &args, &reply)
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
					case <- self.channels.snapshotCh:
						// Trigger Snapshots
					case <- time.After(120 * time.Millisecond):
						self.mu.Lock()
						self.SendHeartBeats()
						self.mu.Unlock()
				}
			case FOLLOWER:
				select {
					case <- self.channels.grantVotes:
					case <- self.channels.heartBeatResp:
					case <- time.After(self.setElectionTime() * time.Millisecond):
						self.electSelf(FOLLOWER)
				}

			case CANDIDATE:
				select {
					case <- self.channels.stepDown:
					case <- self.channels.winElection:
						self.ReinitialisedAfterElection()

					// If the election timer has expired then start a new election 
					case <- time.After(self.setElectionTime() * time.Millisecond):
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
		VoteCount: 0,
		// ElectionTimer: time.Now().Add(SetElectionTime()) ,
	}

	state.Log = append(state.Log, LogEntry{Term: 0})

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
		snapshotCh: make(chan SnapshotCmd),
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
