package mr

import (
	"log"
	"net"
	"net/http"
	"net/rpc"
	"os"
	"sync"
	"time"
)


type MapTasks struct { 
	id int
	file string 
	startAt time.Time
	isDone bool
}

type ReduceTasks struct { 
	id int 
	files []string
	startAt time.Time
	isDone bool 
}


type Coordinator struct {
	locks sync.Mutex
	reduceTasks []ReduceTasks
	mapTasks []MapTasks
	mapLeft int 
	reduceLeft int

}

// Your code here -- RPC handlers for the worker to call.

//
// an example RPC handler.
//
// the RPC argument and reply types are defined in rpc.go.
//
func (c *Coordinator) Example(args *ExampleArgs, reply *ExampleReply) error {
	reply.Y = args.X + 1
	return nil
}


//
// start a thread that listens for RPCs from worker.go
//
func (c *Coordinator) server() {
	rpc.Register(c)
	rpc.HandleHTTP()
	//l, e := net.Listen("tcp", ":1234")
	sockname := coordinatorSock()
	os.Remove(sockname)
	l, e := net.Listen("unix", sockname)
	if e != nil {
		log.Fatal("listen error:", e)
	}
	go http.Serve(l, nil)
}

//
// main/mrcoordinator.go calls Done() periodically to find out
// if the entire job has finished.
//
func (c *Coordinator) Done() bool {
	c.locks.Lock()
	defer c.locks.Unlock()
	return c.mapLeft == 0 && c.reduceLeft == 0
}

//
// create a Coordinator.
// main/mrcoordinator.go calls this function.
// nReduce is the number of reduce tasks to use.
//
func MakeCoordinator(files []string, nReduce int) *Coordinator {
	c := Coordinator{

		mapTasks: make([]MapTasks, len(files)),
		reduceTasks: make([]ReduceTasks, nReduce),
		mapLeft: len(files),
		reduceLeft: nReduce,
	}

	// Your code here.
	
	
	// Intialise Map 
	for idx, file := range files { 
		c.mapTasks[idx] = MapTasks{id: idx, file: file, isDone: false}
	}


	// Initialise Reduce 
	for idx := 0; idx < nReduce; idx +=1 { 
		c.reduceTasks[idx] = ReduceTasks{id: idx, isDone: false}
	}




	c.server()
	return &c
}
