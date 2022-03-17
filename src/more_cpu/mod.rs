pub mod algo;
pub mod measurements;
pub mod types;

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

pub use algo::*;
pub use types::*;

lazy_static::lazy_static! {
    pub static ref ASK_BATCH_QUEUE: Arc<Mutex<Vec<(NodeId, OperationIds)>>> = Default::default();
    pub static ref BATCH_SEND_QUEUE: Arc<Mutex<Vec<(NodeId, OperationIds)>>> = Default::default();
    pub static ref SEND_OPERATION: Arc<Mutex<VecDeque<(NodeId, AskedOperations)>>> = Default::default();
}
