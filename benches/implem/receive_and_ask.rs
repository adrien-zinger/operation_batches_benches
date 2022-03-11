use std::{sync::{atomic::AtomicBool, Arc, Mutex}};
use criterion::Criterion;
use rand::{seq::SliceRandom, Rng};

pub fn less_cpu_receive_and_ask(c: &mut Criterion) {
    use bench_sandbox::less_cpu::*;
    const MAX_BATCH_SIZE: usize = 100;
    const OP_BATCH_PROC_PERIOD: u64 = 200;    
    const T: usize = 25;                  // Numer of nodes
    const N: usize = 10_000;         // Number of operations
    // for this test we need to have the same buffer size as the input
    // operations number
    const OP_BATCH_BUF_CAPACITY: usize = N;
    if N % MAX_BATCH_SIZE != 0 {
        panic!("For the test N should be a multiple of MAX_BATCH_SIZE");
    }
    let mut orders = Box::new([[0; N + 1]; T + 1]);     // Order of operations for each node
    let mut operations = Box::new([0usize; N + 1]); // Table of operations
    for n in 0..N {
        for order in orders.iter_mut() {
            order[n] = n;                                 // init an order to shuffle later
        }
        operations[n] = rand::random();
    }
    let mut thread_rng = rand::thread_rng();
    for order in orders.iter_mut() {
        order.shuffle(&mut thread_rng);
    }
    let protocol = Arc::new(Mutex::new(FakeProtocol::new(T, MAX_BATCH_SIZE, OP_BATCH_PROC_PERIOD, OP_BATCH_BUF_CAPACITY)));
    let protocol_thrd = protocol.clone();
    let running = Arc::new(AtomicBool::new(true));
    let running_thrd = running.clone();
    let batched = Arc::new(Mutex::new(OperationIds::default()));
    let batched_thrd = Arc::new(Mutex::new(OperationIds::default()));

    /* Thread to pop operations asked */
    std::thread::spawn(move || {
        let mut thread_rng = rand::thread_rng();
        while running_thrd.load(std::sync::atomic::Ordering::Relaxed) {
            let operations = {
                let mut guard_op_ids = batched_thrd.lock().unwrap();
                let op_ids = if guard_op_ids.len() > MAX_BATCH_SIZE {
                    let mut op_ids = OperationMap::default();
                    guard_op_ids.iter().take(MAX_BATCH_SIZE).for_each(|id| {
                        op_ids.insert(*id, String::new());
                    });
                    op_ids.iter().for_each(|(id,_)| {
                        guard_op_ids.remove(id);
                    });
                    op_ids
                } else {
                    continue
                };
                op_ids
            };
            
            on_operation_received(thread_rng.gen_range(0..T) as u64, operations, &mut protocol_thrd.lock().unwrap());
            // maybe I need to change the following
            std::thread::sleep(std::time::Duration::from_nanos(20));
        }
    });

    c.bench_function(
        "Less cpu on receive batch with an asker thread",
        |b| b.iter(|| {
            let mut p = 0;
            while p < N {
                for (node_id, order) in orders.iter().enumerate() {
                    let mut batch = OperationIds::default();
                    let mut guard_op_ids = batched.lock().unwrap();
                    for &op_id in order.iter().skip(p).take(MAX_BATCH_SIZE) {
                        batch.insert(op_id as u64);
                        guard_op_ids.insert(op_id as u64);
                    }
                    std::mem::drop(guard_op_ids);
                    // >>>>>>>>>> what we measure
                    on_batch_received(batch, node_id as u64, &mut protocol.lock().unwrap())
                    // <<<<<<<<<<
                }
                p += MAX_BATCH_SIZE;
            }
            running.store(false, std::sync::atomic::Ordering::Relaxed);
        }),
    ); 
}

pub fn more_cpu_receive_and_ask(c: &mut Criterion) {
    use bench_sandbox::more_cpu::*;
    const MAX_BATCH_SIZE: usize = 100;
    const T: usize = 25;                  // Numer of nodes
    const N: usize = 10_000;         // Number of operations
    // for this test we need to have the same buffer size as the input
    // operations number
    if N % MAX_BATCH_SIZE != 0 {
        panic!("For the test N should be a multiple of MAX_BATCH_SIZE");
    }
    let mut orders = Box::new([[0; N + 1]; T + 1]);     // Order of operations for each node
    let mut operations = Box::new([0usize; N + 1]); // Table of operations
    for n in 0..N {
        for order in orders.iter_mut() {
            order[n] = n;                                 // init an order to shuffle later
        }
        operations[n] = rand::random();
    }
    let mut thread_rng = rand::thread_rng();
    for order in orders.iter_mut() {
        order.shuffle(&mut thread_rng);
    }
    let protocol = Arc::new(Mutex::new(FakeProtocol::new(T, MAX_BATCH_SIZE)));
    let protocol_thrd = protocol.clone();
    let running = Arc::new(AtomicBool::new(true));
    let running_thrd = running.clone();
    let batched = Arc::new(Mutex::new(OperationIds::default()));
    let batched_thrd = Arc::new(Mutex::new(OperationIds::default()));


    /* Thread to pop operations asked */
    std::thread::spawn(move || {
        let mut thread_rng = rand::thread_rng();
        while running_thrd.load(std::sync::atomic::Ordering::Relaxed) {
            let operations = {
                let mut guard_op_ids = batched_thrd.lock().unwrap();
                let op_ids = if guard_op_ids.len() > MAX_BATCH_SIZE {
                    let mut op_ids = AskedOperations::default();
                    guard_op_ids.iter().take(MAX_BATCH_SIZE).for_each(|id| {
                        op_ids.insert(*id, Some(String::new()));
                    });
                    op_ids.iter().for_each(|(id,_)| {
                        guard_op_ids.remove(id);
                    });
                    op_ids
                } else {
                    continue
                };
                op_ids
            };
            on_operation_received(thread_rng.gen_range(0..T) as u64, operations, &mut protocol_thrd.lock().unwrap());
            // maybe I need to change the following
            std::thread::sleep(std::time::Duration::from_nanos(20));
        }
    });

    c.bench_function(
        "More cpu on receive batch with an asker thread",
        |b| b.iter(|| {
            let mut p = 0;
            while p < N {
                for (node_id, order) in orders.iter().enumerate() {
                    let mut batch = OperationIds::default();
                    let mut guard_op_ids = batched.lock().unwrap();
                    for &op_id in order.iter().skip(p).take(MAX_BATCH_SIZE) {
                        batch.insert(op_id as u64);
                        guard_op_ids.insert(op_id as u64);
                    }
                    std::mem::drop(guard_op_ids);
                    // >>>>>>>>>> what we measure
                    on_batch_received(batch, node_id as u64, &mut protocol.lock().unwrap())
                    // <<<<<<<<<<
                }
                p += MAX_BATCH_SIZE;
            }
            running.store(false, std::sync::atomic::Ordering::Relaxed);
        }),
    ); 
}