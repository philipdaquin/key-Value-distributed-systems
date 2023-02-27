package kvraft

import (
	"crypto/rand"
	// "fmt"
	"log"
	"math/big"

	"6.5840/labrpc"
)


type Clerk struct {
	servers []*labrpc.ClientEnd
	// You will have to modify this struct.
	ClientId int 
	RequestId int
	LeaderId int 
}

func nrand() int64 {
	max := big.NewInt(int64(1) << 62)
	bigx, _ := rand.Int(rand.Reader, max)
	x := bigx.Int64()
	return x
}

func MakeClerk(servers []*labrpc.ClientEnd) *Clerk {
	ck := new(Clerk)
	ck.servers = servers
	// You'll have to add code here.

	ck.ClientId = int(nrand())
	ck.LeaderId = 0
	ck.RequestId = 0

	return ck
}

// fetch the current value for a key.
// returns "" if the key does not exist.
// keeps trying forever in the face of all other errors.
//
// you can send an RPC with code like this:
// ok := ck.servers[i].Call("KVServer.Get", &args, &reply)
//
// the types of args and reply (including whether they are pointers)
// must match the declared types of the RPC handler function's
// arguments. and reply must be passed as a pointer.
func (ck *Clerk) Get(key string) string {
	leader :=  ck.LeaderId
	requestId := ck.RequestId + 1

	// You will have to modify this function.
	log.Println("📦 Getting Value from a key")
	getArgs := GetArgs{
		Key: key,
		ClientId: ck.ClientId,
		RequestId: requestId,
	}
	var getReply GetReply
	var value string 

	server := leader
	for { 
		ok := ck.servers[server].Call("KVServer.Get", &getArgs, &getReply)

		if !ok && getReply.Err != ErrWrongLeader { 

			if getReply.Err == OK {
				value = getReply.Value
			}
			break
		}

		log.Println("Got the value, found the leader at", server)
		server = (server + 1) & len(ck.servers)
	}

	return value
}

// shared by Put and Append.
//
// you can send an RPC with code like this:
// ok := ck.servers[i].Call("KVServer.PutAppend", &args, &reply)
//
// the types of args and reply (including whether they are pointers)
// must match the declared types of the RPC handler function's
// arguments. and reply must be passed as a pointer.
func (ck *Clerk) PutAppend(key string, value string, op string) {
	// You will have to modify this function.

	leader  := ck.LeaderId
	log.Println("🚀 PUTTING Value from a key")

	putArgs := PutAppendArgs{
		Key: key,
		Value: value,
		Op: op,
		ClientId: ck.ClientId,
		RequestId: ck.RequestId + 1,
	}
	var putReply PutAppendReply
	var reply PutAppendReply

	server := leader

	for  {
		ok := ck.servers[server].Call("KVServer.PutAppend", &putArgs, &putReply)

		if ok && reply.Err != ErrWrongLeader {
			break
		}

		server = (server + 1) & len(ck.servers)
	}
}

func (ck *Clerk) Put(key string, value string) {
	ck.PutAppend(key, value, "Put")
}
func (ck *Clerk) Append(key string, value string) {
	ck.PutAppend(key, value, "Append")
}
