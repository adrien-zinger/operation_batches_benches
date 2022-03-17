use super::types::*;
use std::time::{Duration, Instant};

/***************************************************************************************** */
/* Things that must be in the both algorithms                                              */
// - send_batch(node_id, operations_ids): Send batch to a node
// - on_batch_received(): Receive a batch of operation ids
// - ask_operations(node_id, operations_ids): Ask operations to a node, on_ask_received
// - on_ask_received ...
// - send_operations(node_id, operations_ids): Ask operations to a node
// - on_operation_received ...
/* *************************************************************************************** */

fn send_batch(_to_node_id: NodeId, _batch: OperationIds) {
    //#[cfg(feature = "measurements")]
    super::BATCH_SEND_QUEUE
        .lock()
        .unwrap()
        .push((_to_node_id, _batch));
}

fn ask_operations(_to_node_id: NodeId, _op_ids: OperationIds) {
    //#[cfg(feature = "measurements")]
    super::ASK_BATCH_QUEUE
        .lock()
        .unwrap()
        .push_back((_to_node_id, _op_ids));
}

// difer from the other algo
fn send_operations(_to_node_id: NodeId, _op_ids: OperationMap) {
    //#[cfg(feature = "measurements")]
    super::SEND_OPERATION
        .lock()
        .unwrap()
        .push_back((_to_node_id, _op_ids));
}

///```py
///def process_op_batch(op_batch, node_id):
///    ask_set = void HashSet<OperationId>
///    future_set = void HashSet<OperationId>
///    for op_id in op_batch:
///        if not is_op_received(op_id):
///            if (op_id not in asked_ops) or (node_id not in asked_ops(op_id)[1]):
///                if (op_id not in asked_ops) or (asked_ops(op_id)[0] < now - op_batch_proc_period:
///                    ask_set.add(op_id)
///                    asked_ops(op_id)[0] = now
///                    asked_ops(op_id)[1].add(node_id)
///                else:
///                    future_set.add(op_id)
///    if op_batch_buf is not full:
///        op_batch_buf.push(now+op_batch_proc_period, node_id, future_set)
///    ask ask_set to node_id
///```
///
/// # Return
///
/// Operation asked in that call (usefull for measurement but not needed in
/// the final implementation)
pub fn on_batch_received(
    op_batch: OperationIds,
    node_id: NodeId,
    protocol: &mut FakeProtocol, /* self simulation */
) -> OperationIds {
    let mut ask_set = OperationIds::with_capacity(op_batch.len());
    let mut future_set = OperationIds::with_capacity(op_batch.len());
    // exactitude isn't important, we want to have a now for that function call
    let now = Instant::now();
    for op_id in op_batch {
        if protocol.received.contains_key(&op_id) {
            // Should I manage here the prune of `wanted`, `op_batch_buffer` etc?
            continue;
        }
        let wish = match protocol.wanted_alias_asked_ops.get(&op_id) {
            Some(wish) => {
                if wish.1.contains(&node_id) {
                    continue; // already asked to the `node_id`
                } else {
                    Some(wish)
                }
            }
            None => None,
        };
        if wish.is_some() && wish.unwrap().0 > now {
            future_set.insert(op_id);
        } else {
            ask_set.insert(op_id);
            protocol
                .wanted_alias_asked_ops
                .insert(op_id, (now, vec![node_id]));
        }
    }
    if protocol.op_batch_buffer.len() < protocol.op_batch_buf_capacity {
        protocol.op_batch_buffer.push_back((
            now + Duration::from_millis(protocol.op_batch_proc_period),
            node_id,
            future_set,
        ));
    }
    if protocol.is_measured {
        // just for the measurement, remove that on the definitive implementation
        ask_operations(node_id, ask_set.clone());
    }
    ask_set
}

/* We can prune the buffer from the informations received in another future.
 */

pub fn on_operation_received(
    node_id: NodeId,
    operations: OperationMap,
    protocol: &mut FakeProtocol, /* self simulation */
) {
    protocol.received.extend(operations.clone());
    if let Some(node_info) = protocol.node_infos.get_mut(&node_id) {
        node_info.known_op.extend(operations.keys());
    }
    for (node_id, node_info) in protocol.node_infos.iter_mut() {
        let mut batch = OperationIds::default();
        batch.extend(
            operations
                .keys()
                .filter(|&&op_id| node_info.known_op.insert(op_id)),
        );
        if protocol.is_measured {
            // just for the measurement, remove that on the definitive implementation
            send_batch(*node_id, batch);
        }
    }
}

pub fn on_ask_received(
    node_id: NodeId,
    op_ids: OperationIds,
    protocol: &mut FakeProtocol, /* self simulation */
) {
    if let Some(node_info) = protocol.node_infos.get_mut(&node_id) {
        for op_ids in op_ids.iter() {
            node_info.known_op.remove(op_ids);
        }
    }
    let mut operation_map = OperationMap::default();
    for op_id in op_ids.iter() {
        if let Some(op) = protocol.received.get(op_id) {
            operation_map.insert(*op_id, op.clone());
        }
    }
    if protocol.is_measured {
        // just for the measurement, remove that on the definitive implementation
        send_operations(node_id, operation_map);
    }
}

/// Take the op_batch_buffer and reprocess on batch received
pub fn on_send_loop(protocol: &mut FakeProtocol /* self simulation */) {
    while !protocol.op_batch_buffer.is_empty()
        && std::time::Instant::now() > protocol.op_batch_buffer.front().unwrap().0
    {
        let (_, node_id, op_batch) = protocol.op_batch_buffer.pop_front().unwrap();
        on_batch_received(op_batch, node_id, protocol);
    }
}

/// Should be done in a loop each `asked_life_time` period
///
/// (let's don't prune for now, we will absolutly do that in the final implementation)
pub fn _on_prune_asked_lifetime_loop(protocol: &mut FakeProtocol /* self simulation */) {
    protocol.wanted_alias_asked_ops.clear();
}
