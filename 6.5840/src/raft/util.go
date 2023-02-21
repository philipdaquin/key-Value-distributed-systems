package raft

import (
	"log"
	"time"
	"math/rand"

)

// Debugging
const Debug = false

func DPrintf(format string, a ...interface{}) (n int, err error) {
	if Debug {
		log.Printf(format, a...)
	}
	return
}


// Set normal election timeout, with randomness
func ResetElectionTime() time.Time {

	randomRange := rand.Intn(MaxElectionTimeOut - MinElectionTimeOut + 1) + MinElectionTimeOut 

	return time.Now().Add(time.Duration(randomRange))
}