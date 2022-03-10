use std::collections::{HashMap, HashSet};

pub type OperationId = u64;
pub type NodeId = u64;
pub type Operation = String;
pub type OperationMap = HashMap<OperationId, Operation>;
pub type OperationIds = HashSet<OperationId>;

pub type AskedOperations = std::collections::HashMap<OperationId, Option<Operation>>;
/// Internal data structure describing the [Operation] we do want from which `NodeId`.
pub type WantOperations = std::collections::HashMap<NodeId, HashSet<OperationId>>;
/// Same as wanted operation but used to propagate `OperationId` through `NodeId`
pub type OperationBatches = std::collections::HashMap<NodeId, Vec<OperationId>>;
/// just a list of operation required
pub type OperationBatch = Vec<OperationId>;

#[derive(Default)]
pub struct NodeInfo {
    pub known_op: OperationIds,
    pub wishlist: OperationIds,
}

pub struct FakeProtocol {
    /// Remember that nodes know and have
    pub node_infos: HashMap<NodeId, NodeInfo>,
    /// list of operation that the node wish
    pub wishlist: OperationIds,
    /// Wishlist converted to route to a specific NodeId
    pub wanted: WantOperations,
    /// List of all operations that we already asked for
    pub already_asked: OperationIds,
    /// Map<OperationId, Operation> received!
    pub received: OperationMap,

    /// config maximum size of a batch (number of operations)
    pub max_batch_size: usize,
}
