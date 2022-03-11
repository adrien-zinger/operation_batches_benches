use std::{
    collections::{HashMap, HashSet, VecDeque},
    time::Instant,
};

pub type OperationId = u64;
pub type NodeId = u64;
pub type Operation = String;
pub type OperationMap = HashMap<OperationId, Operation>;
pub type OperationIds = HashSet<OperationId>;

/// Data structure forwarded in the network after asking [Operation].
/// Option is None if the asked node hasn't the operation.
pub type AskedOperations = HashMap<OperationId, (Instant, HashSet<NodeId>)>;
/* ****  Following Difer from the algo A **** */
/// Internal data structure describing the [Operation] we do want from which `NodeId`.
pub type WantOperations = HashMap<OperationId, (Instant, Vec<NodeId>)>;
/// Same as wanted operation but used to propagate `OperationId`
pub type OperationBatch = Vec<OperationId>;

#[derive(Default)]
pub struct NodeInfo {
    pub known_op: OperationIds,
    pub wishlist: OperationIds,
}

pub struct FakeProtocol {
    pub node_infos: HashMap<NodeId, NodeInfo>,
    /// Wishlist converted to route to a specific NodeId
    pub wanted_alias_asked_ops: WantOperations,
    /// Map<OperationId, Operation> received!
    pub received: OperationMap,

    /* Specific structure for the algorithm */
    /// Buffer for operations that we want later
    pub op_batch_buffer: VecDeque<(Instant, NodeId, HashSet<OperationId>)>,
    /* following should be in a configuration object */
    /// config max_batch_size
    pub max_batch_size: usize,
    /// config operation_period
    pub op_batch_proc_period: u64,
    /// config buffer capacity limit [FakeProtocol::op_batch_buffer]
    pub op_batch_buf_capacity: usize,
}

impl FakeProtocol {
    pub fn new(
        nodes_number: usize,
        max_batch_size: usize,
        op_batch_proc_period: u64,
        op_batch_buf_capacity: usize
    ) -> Self {

        let mut node_infos = HashMap::default();
        for k in 0..nodes_number {
            node_infos.insert(k as u64, NodeInfo::default());
        }
        Self {
            node_infos,
            wanted_alias_asked_ops: WantOperations::default(),
            received: OperationMap::default(),
            op_batch_buffer: VecDeque::default(),
            max_batch_size,
            op_batch_proc_period,
            op_batch_buf_capacity,
        }
    }
}