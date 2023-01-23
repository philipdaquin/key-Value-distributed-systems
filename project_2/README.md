# The Design and Implementation of a Log Structured File System 
An LFS is a type of file system that organises data as a log, rather than as a traditional file hierarchy. In LFS, all writes are appended to a single log file, which is called a "Write Ahead Log" (WAL). The log file is then periodically "checkpointed" to create a new version of the file system that can be quickly restored in case of a crash

## Log-Structure file I/O
Persistent key / Value store that can be accessed from the command line 

## Technical Specs:
The cargo project, kvs, builds a command-line key-value store client called kvs, which in turn calls into a library called kvs.

The kvs executable supports the following command line arguments:

`kvs set <KEY> <VALUE>`

Set the value of a `K` key to a `V`. Print an error and return a non-zero exit code on failure

`kvs get <KEY>`

Get the `V` value of a given `K` key. Print an error and return a non-zero exit code on failure 

`kvs rm <KEY>`

Remove a given key. Print an error and return a non-zero exit code on failure

`kvs -V`

Print the version

The kvs library contains a type, KvStore, that supports the following methods:

`KvStore::set(&mut self, key: String, value: String)`

Set the value of a `V` key to a `K`. Return an error if the value is not written successfully

`KvStore::get(&self, key: String) -> Option<String>`

Get the `V` value of the a `K` key. If the key does not exist, return None.

`KvStore::remove(&mut self, key: String)`

Remove a given key.

`KvStore::open(path: Impl Into<PathBuf>) -> Result<KvStore>`

Open the KvStore at a given path. Return the KvStore

When setting a key to a value, kvs writes the set command to disk in a sequential log, then stores the log pointer (file offset) of that command in the in-memory index from key to pointer. When removing a key, similarly, kvs writes the rm command in the log, then removes the key from the in-memory index. When retrieving a value for a key with the get command, it searches the index, and if found then loads from the log the command at the corresponding log pointer, evaluates the command and returns the result.

On startup, the commands in the log are traversed from oldest to newest, and the in-memory index rebuilt.

When the size of the uncompacted log entries reach a given threshold, kvs compacts it into a new log, removing redundent entries to reclaim disk space.

Note that our kvs project is both a stateless command-line program, and a library containing a stateful KvStore type: for CLI use the KvStore type will load the index, execute the command, then exit; for library use it will load the index, then execute multiple commands, maintaining the index state, until it is dropped.

## Optimisations 
Two optimisations can be done to fit more data:
- Data compression 
- Store only frequently used data in memory and the rest of disk (fitting everything in memory may be impossible)