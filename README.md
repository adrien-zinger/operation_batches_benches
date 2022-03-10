# Operation batch quest

This repository is used for leading a technical specification for the implementation of an optimization in a P2P network in the massalabs blockchain.

In the sources you can find a _less_cpu_ and a _more_cpu_ folder, each one implementing a different algorithm in a subfile `algo.rs` that follow the following mandatory functions:

```rust
/*************************************************************************** */
/* Things that must be in the both algorithms                                */
// - send_batch(node_id, operations_ids): Send batch to a node
// - on_batch_received(): Receive a batch of operation ids
// - ask_operations(node_id, operations_ids): Ask operations to a node,
//   on_ask_received
// - on_ask_received ...
// - send_operations(node_id, operations_ids): Ask operations to a node
// - on_operation_received ...
/* ************************************************************************* */
```

And more if needed. The datastructures used are declared in a `types.rs`

The main function (todo) let you choose with algorithm to run with a predetermined scenario that can be repeted indefinitively.

## Scenario description

**_TODO_**
