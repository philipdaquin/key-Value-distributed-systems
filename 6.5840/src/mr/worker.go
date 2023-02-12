package mr

import (
	"encoding/json"
	"fmt"
	"hash/fnv"
	"io/ioutil"
	"log"
	"net/rpc"
	"os"
)

//
// Map functions return a slice of KeyValue.
//
type KeyValue struct {
	Key   string
	Value string
}

//
// use ihash(key) % NReduce to choose the reduce
// task number for each KeyValue emitted by Map.
//
func ihash(key string) int {
	h := fnv.New32a()
	h.Write([]byte(key))
	return int(h.Sum32() & 0x7fffffff)
}

//
// main/mrworker.go calls this function.
//
// 
func Worker(mapf func(string, string) []KeyValue,
	reducef func(string, []string) string) {

	// Your worker implementation here.


	coordinator := "Coordinator"
	args := TaskArgs{}
	reply := TaskReply{}
	

	if ok := call(coordinator, &args, &reply); ok {
		fmt.Println("message went through")
	} else { 
		fmt.Println("Something went wrong")
	}
	// uncomment to send the Example RPC to the coordinator.
	// CallExample()
}

func GetNextTask(completed TaskArgs) TaskReply {
	nextTask := TaskReply{}

	if ok := call("Coordinator.GetNextTask", &completed, &nextTask); !ok {
		fmt.Println("Failed to get response")
		os.Exit(1)
	}
	return nextTask
}

func MapTask(
	nextTask TaskReply, 
	mapf func(string, string) []KeyValue) {
	task := nextTask.ImpendingTasks[0]

	file, err := os.Open(task)
	
	defer file.Close()

	if err != nil { panic(err) }

	content, err := ioutil.ReadAll(file)

	kvs := mapf(task, string(content))

	reducedFiles := make(map[int][]KeyValue)
	
	for _, kv := range kvs { 
		index := ihash(kv.Key) % nextTask.NReduce
		reducedFiles[index] = append(reducedFiles[index], kv)
	}

	files := make([]string, nextTask.NReduce)

	for idx, kvFiles := range reducedFiles { 
		
		fileName := fmt.Sprintf("mr-%d-%d", nextTask.WorkerId, idx)

		newFile, _ := os.Create(fileName)

		defer newFile.Close()

		encoded := json.NewEncoder(newFile)

		for _, kv := range kvFiles { 
			if err := encoded.Encode(&kv); err != nil { 
				panic(err)
			}
		}
		files[idx] = fileName
	}
}



//
// example function to show how to make an RPC call to the coordinator.
//
// the RPC argument and reply types are defined in rpc.go.
//
func CallExample() {

	// declare an argument structure.
	args := ExampleArgs{}

	// fill in the argument(s).
	args.X = 99

	// declare a reply structure.
	reply := ExampleReply{}

	// send the RPC request, wait for the reply.
	// the "Coordinator.Example" tells the
	// receiving server that we'd like to call
	// the Example() method of struct Coordinator.
	ok := call("Coordinator.Example", &args, &reply)
	if ok {
		// reply.Y should be 100.
		fmt.Printf("reply.Y %v\n", reply.Y)
	} else {
		fmt.Printf("call failed!\n")
	}
}

//
// send an RPC request to the coordinator, wait for the response.
// usually returns true.
// returns false if something goes wrong.
//
func call(rpcname string, args interface{}, reply interface{}) bool {
	// c, err := rpc.DialHTTP("tcp", "127.0.0.1"+":1234")
	sockname := coordinatorSock()
	c, err := rpc.DialHTTP("unix", sockname)
	if err != nil {
		log.Fatal("dialing:", err)
	}
	defer c.Close()

	err = c.Call(rpcname, args, reply)
	if err == nil {
		return true
	}

	fmt.Println(err)
	return false
}
