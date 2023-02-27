package kvraft

import (
	"log"
	"sync"
	"sync/atomic"
	"time"

	"6.5840/labgob"
	"6.5840/labrpc"
	"6.5840/raft"
)

const Debug = false

func DPrintf(format string, a ...interface{}) (n int, err error) {
	if Debug {
		log.Printf(format, a...)
	}
	return
}


type Op struct {
	// Your definitions here.
	// Field names must start with capital letters,
	// otherwise RPC will break.
	Type string
	Key string 
	Value string 

	ClientId int
	RequestId int 
}

type KVServer struct {
	mu      sync.Mutex
	me      int
	rf      *raft.Raft
	applyCh chan raft.ApplyMsg
	dead    int32 // set by Kill()

	maxraftstate int // snapshot if log grows this big

	// Your definitions here.
	result map[int]chan Op
	store map[string]string
	lastApplied map[int]int
}



func (kv *KVServer) Get(args *GetArgs, reply *GetReply) {
	// Your code here.
	log.Println("📞 Received Message, getting the value")

	operation := Op{
		Type: "Get",
		Key: args.Key,
		ClientId: args.ClientId,
		RequestId: args.RequestId,
	}

	res, op := kv.sendMessage(operation)

	if !res { 
		reply.Err = ErrNoKey
		return 
	}

	reply.Value = op.Value
	reply.Err = OK
}

func (kv *KVServer) PutAppend(args *PutAppendArgs, reply *PutAppendReply) {
	log.Println("😪 Adding value")

	res, _ := kv.sendMessage(Op{
		Type: "Put",
		Key: args.Key,
		Value: args.Value,
		ClientId: args.ClientId,
		RequestId: args.RequestId,
	})

	if !res {
		reply.Err = ErrWrongLeader
		return 
	}

	reply.Err = OK

}

func (self *KVServer) backgroundApply() {

	log.Println("✅ BackgroundApply")

	for {
		applyMsg := <- self.applyCh

		if !applyMsg.CommandValid { continue }

		index := applyMsg.CommandIndex
		command := applyMsg.Command.(Op)

		self.mu.Lock()


		if command.Type == "Get" {
			command.Value = self.store[command.Key]
		} else { 
			id, ok := self.lastApplied[command.ClientId]

			if !ok  || command.RequestId > id { 
				self.applyMessage(&command)
				self.lastApplied[command.ClientId] = command.RequestId
			}
		}
		commandCh, ok := self.result[index]
		if !ok { 
			commandCh = make(chan Op, 1)

			self.result[index] = commandCh
		} 
		
		commandCh <- command
		
		self.mu.Unlock()
	}
}


func (self *KVServer) applyMessage(op *Op) { 
	switch op.Type { 
	case "Get":
		op.Value = self.store[op.Key]
	case "Set":
		self.store[op.Key] = op.Value
	case "Append":
		self.store[op.Key] += op.Value
	}
}


// the tester calls Kill() when a KVServer instance won't
// be needed again. for your convenience, we supply
// code to set rf.dead (without needing a lock),
// and a killed() method to test rf.dead in
// long-running loops. you can also add your own
// code to Kill(). you're not required to do anything
// about this, but it may be convenient (for example)
// to suppress debug output from a Kill()ed instance.
func (kv *KVServer) Kill() {
	atomic.StoreInt32(&kv.dead, 1)
	kv.rf.Kill()
	// Your code here, if desired.
}

func (kv *KVServer) killed() bool {
	z := atomic.LoadInt32(&kv.dead)
	return z == 1
}

func (kv *KVServer) sendMessage(operation Op) (bool, Op) {
	index, _, isLeader := kv.rf.Start(operation)

	// 
	if !isLeader { 
		return false, operation
	} 

	kv.mu.Lock()

	var res chan Op
	if val, ok := kv.result[index]; !ok { 
		kv.result[index] = make(chan Op, 1)
		res = val
	}
	kv.mu.Unlock()

	select { 

	case applied := <- res:
		return kv.isSameOp(operation , applied), applied
	
	case <- time.After(600 * time.Millisecond):
		return false, operation
	
	}
}

//
// check if the issued command is the same as the applied command
//
func (kv *KVServer) isSameOp(issued Op, applied Op) bool {
	return issued.ClientId == applied.ClientId &&
		issued.RequestId == applied.RequestId
}

// servers[] contains the ports of the set of
// servers that will cooperate via Raft to
// form the fault-tolerant key/value service.
// me is the index of the current server in servers[].
// the k/v server should store snapshots through the underlying Raft
// implementation, which should call persister.SaveStateAndSnapshot() to
// atomically save the Raft state along with the snapshot.
// the k/v server should snapshot when Raft's saved state exceeds maxraftstate bytes,
// in order to allow Raft to garbage-collect its log. if maxraftstate is -1,
// you don't need to snapshot.
// StartKVServer() must return quickly, so it should start goroutines
// for any long-running work.
func StartKVServer(servers []*labrpc.ClientEnd, me int, persister *raft.Persister, maxraftstate int) *KVServer {
	// call labgob.Register on structures you want
	// Go's RPC library to marshall/unmarshall.
	labgob.Register(Op{})

	kv := new(KVServer)
	kv.me = me
	kv.maxraftstate = maxraftstate

	// You may need initialization code here.

	kv.applyCh = make(chan raft.ApplyMsg, 1)
	kv.rf = raft.Make(servers, me, persister, kv.applyCh)

	// You may need initialization code here.
	kv.result = make(map[int]chan Op)
	kv.store=  make(map[string]string)
	kv.lastApplied= make(map[int]int)
	go kv.backgroundApply()

	return kv
}
