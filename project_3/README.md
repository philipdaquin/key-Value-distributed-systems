# Synchronous Key / Value Server and Client 
Create a single threaded, persistent key / value store server and client with synchronous networking over a custom protocol 

## Project spec
The cargo project, `kvs`, builds a command-line key-value store client called `kvs-client`, and a key-value store server called `kvs-server`, both of which in turn call into a library called `kvs`. The client speaks to the server over a custom protocol.

The `kvs-server` executable supports the following command line arguments:

- `kvs-server [--addr IP-PORT] [--engine ENGINE-NAME]`

Start the server and begin listening for incoming connections. `--addr` accepts an IP address, either v4 or v6, and a port number, with the format `IP:PORT`. If `--addr` is not specified then listen on `127.0.0.1:4000`.

If `--engine` is specified, then `ENGINE-NAME` must be either "kvs", in which case the built-in engine is used, or "sled", in which case sled is used. If this is the first run (there is no data previously persisted) then the default value is "kvs"; if there is previously persisted data then the default is the engine already in use. If data was previously persisted with a different engine than selected, print an error and exit with a non-zero exit code.

Print an error and return a non-zero exit code on failure to bind a socket, if `ENGINE-NAME` is invalid, if IP-PORT does not parse as an address.

- `kvs-server -V`

Print the version.

The `kvs-client` executable supports the following command line arguments:

- `kvs-client set <KEY> <VALUE> [--addr IP-PORT]`

Set the value of a string key to a string.

`--addr` accepts an IP address, either v4 or v6, and a port number, with the format `IP:PORT`. If `--addr` is not specified then connect on `127.0.0.1:4000`.

Print an error and return a non-zero exit code on server error, or if IP-PORT does not parse as an address.

- `kvs-client get <KEY> [--addr IP-PORT]`

Get the string value of a given string key.

`--addr` accepts an IP address, either v4 or v6, and a port number, with the format `IP:PORT`. If `--addr` is not specified then connect on `127.0.0.1:4000`.

Print an error and return a non-zero exit code on server error, or if IP-PORT does not parse as an address.

- `kvs-client rm <KEY> [--addr IP-PORT]`

Remove a given string key.

`--addr` accepts an IP address, either v4 or v6, and a port number, with the format `IP:PORT`. If `--addr` is not specified then connect on `127.0.0.1:4000`.

Print an error and return a non-zero exit code on server error, or if IP-PORT does not parse as an address. A "key not found" is also treated as an error in the "rm" command.

- `kvs-client -V`

Print the version.

All error messages should be printed to stderr.

The `kvs` library contains four types:

- `KvsClient` - implements the functionality required for kvs-client to speak to kvs-server
- `KvsServer` - implements the functionality to serve responses to kvs-client from kvs-server
- `KvsEngine` trait - defines the storage interface called by KvsServer
- `KvStore` - implements by hand the KvsEngine trait
- `SledKvsEngine` - implements KvsEngine for the sled storage engine.

The design of `KvsClient` and `KvsServer` are up to you, and will be informed by the design of your network protocol. The test suite does not directly use either type, but only exercises them via the CLI.

The `KvsEngine` trait supports the following methods:

- `KvsEngine::set(&mut self, key: String, value: String) -> Result<()>`

Set the value of a string key to a string.

Return an error if the value is not written successfully.

- `KvsEngine::get(&mut self, key: String) -> Result<Option<String>>`

Get the string value of a string key. If the key does not exist, return `None`.

Return an error if the value is not read successfully.

- `KvsEngine::remove(&mut self, key: String) -> Result<()>`

Remove a given string key.

Return an error if the key does not exit or value is not read successfully.

When setting a key to a value, `KvStore` writes the set command to disk in a sequential log. When removing a key, `KvStore` writes the `rm` command to the log. On startup, the commands in the log are re-evaluated and the log pointer (file offset) of the last command to set each key recorded in the in-memory index.

When retrieving a value for a key with the `get` command, it searches the index, and if found then loads from the log, and evaluates, the command at the corresponding log pointer.

When the size of the uncompacted log entries reach a given threshold, `KvStore` compacts it into a new log, removing redundant entries to reclaim disk space.