# Project 4: Concurrency and Parallelism
Task: Create a multithreaded, persistent key/value store and client with synchrononous networking over a customer protocol

## Multi-threaded requirements
- Read from the index and from the index, on multiple threads at a time
- Write comamnds to disk, while maintaining the index
- Read in parallel with writing
- In general, to guarantee that readers will always see a consistent state while reading in parallel with a writer which means,
    - Maintaining an invariant that log pointers in the index always point to a valid command in the log,
    - Maintaining appropriate invariants for other bookkeeping, like the `uncompacted` variable in the following example
- Periodically compact our on disk data, again while maintaining invariants for readers