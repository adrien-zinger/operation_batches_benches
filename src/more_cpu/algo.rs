use super::types::*;

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
    std::thread::sleep(std::time::Duration::from_nanos(200))
}

fn ask_operations(_to_node_id: NodeId, _op_ids: OperationIds) {
    std::thread::sleep(std::time::Duration::from_nanos(200))
}

fn send_operations(_to_node_id: NodeId, _op_ids: AskedOperations) {
    // difer from the other algo
    std::thread::sleep(std::time::Duration::from_nanos(200))
}

fn on_batch_received(
    op_batch: OperationBatch,
    node_id: NodeId,
    protocol: &mut FakeProtocol, /* self simulation */
) {
    for op_id in op_batch {
        if protocol.received.contains_key(&op_id) {
            continue;
        }
        let node_info_opt = protocol.node_infos.get_mut(&node_id);
        match node_info_opt {
            Some(node_info) => {
                node_info.known_op.insert(op_id);
            }
            None => {
                let mut info = NodeInfo::default();
                info.known_op.insert(op_id);
                protocol.node_infos.insert(node_id, info).unwrap();
            }
        }
        protocol.wishlist.insert(op_id);
    }
}

fn on_asking_loop(protocol: &mut FakeProtocol /* self simulation */) {
    for op_id in protocol.wishlist.iter() {
        if protocol.already_asked.contains(op_id) {
            // insert logic of retry after a while here
            continue;
        }
        if protocol.received.contains_key(op_id) {
            continue;
        }
        // Choose a node that know the operation. Can evolve.
        for (node_id, node_info) in protocol.node_infos.iter() {
            if node_info.known_op.contains(op_id) {
                match protocol.wanted.get_mut(node_id) {
                    Some(op_ids) => {
                        if op_ids.len() < protocol.max_batch_size {
                            protocol.already_asked.insert(*op_id); // Should I limit the wanted object?
                            op_ids.insert(*op_id);
                            break;
                        }
                    }
                    None => {
                        let mut set = OperationIds::default();
                        set.insert(*op_id);
                        protocol.wanted.insert(*node_id, set); // Also here, should I limit the wanted object?
                    }
                };
            }
        }
    }
    for (node_id, wanted) in protocol.wanted.iter() {
        ask_operations(*node_id, wanted.clone());
    }
}

/*  for a potential limitation of the wanted object I can remove the one that has
   always less op_id than the others.
*/

/// Notify operation and forward batches
fn on_operation_received(
    from_node_id: NodeId,
    asked_operation: AskedOperations,
    protocol: &mut FakeProtocol, /* self simulation */
) {
    let op_ids: OperationIds = asked_operation
        .iter()
        .filter(|(_, opt)| opt.is_some())
        .map(|(op_id, operation)| {
            protocol.received.insert(*op_id, operation.clone().unwrap());
            protocol.wishlist.remove(op_id);
            for (_, list) in protocol.wanted.iter_mut() {
                list.remove(op_id);
            }
            *op_id
            // should I also prune the node_infos here?
        })
        .collect();
    if let Some(info) = protocol.node_infos.get_mut(&from_node_id) {
        info.known_op.extend(op_ids.clone());
    }
    for (node_id, node_info) in protocol.node_infos.iter_mut() {
        let batch: OperationIds = op_ids
            .iter()
            .filter(|&&op_id| node_info.known_op.insert(op_id))
            .cloned()
            .collect();
        send_batch(*node_id, batch);
    }
}

/*  It might be better to prune the node_infos in another futures that is also timed
*/

fn on_ask_received(
    node_id: NodeId,
    op_ids: OperationIds,
    protocol: &mut FakeProtocol, /* self simulation */
) {
    match protocol.node_infos.get_mut(&node_id) {
        Some(info) => info.wishlist.extend(op_ids),
        None => {
            // Is there a node info limiation?
            let mut info = NodeInfo::default();
            info.wishlist.extend(op_ids);
            protocol.node_infos.insert(node_id, info);
        }
    }
}

/// Call on `send` timer?
fn on_send_operation_loop(protocol: &mut FakeProtocol) {
    for (node_id, node_info) in protocol.node_infos.iter() {
        let mut asked = AskedOperations::default();
        node_info
            .wishlist
            .clone()
            .drain()
            .take(protocol.max_batch_size)
            .for_each(|op_id| {
                match protocol.received.get(&op_id) {
                    Some(operation) => asked.insert(op_id, Some(operation.clone())),
                    None => asked.insert(op_id, None),
                };
            });
        send_operations(*node_id, asked);
    }
}
