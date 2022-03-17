use criterion::Criterion;
use rand::seq::SliceRandom;

/// Measure when we keep receiving without locks
pub fn less_cpu_simple_receive(c: &mut Criterion) {
    use bench_sandbox::less_cpu::*;

    const MAX_BATCH_SIZE: usize = 100;
    const OP_BATCH_PROC_PERIOD: u64 = 200;

    const T: usize = 25; // Numer of nodes
    const N: usize = 10_000; // Number of operations

    // for this test we need to have the same buffer size as the input
    // operations number
    const OP_BATCH_BUF_CAPACITY: usize = N;

    if N % MAX_BATCH_SIZE != 0 {
        panic!("For the test N should be a multiple of MAX_BATCH_SIZE");
    }

    let mut orders = Box::new([[0; N + 1]; T + 1]); // Order of operations for each node
    let mut operations = Box::new([0usize; N + 1]); // Table of operations
    for n in 0..N {
        for order in orders.iter_mut() {
            order[n] = n; // init an order to shuffle later
        }
        operations[n] = rand::random();
    }
    let mut thread_rng = rand::thread_rng();
    for order in orders.iter_mut() {
        order.shuffle(&mut thread_rng);
    }

    c.bench_function("Less cpu on receive batch", |b| {
        b.iter(|| {
            let mut p = 0;
            let mut protocol = FakeProtocol::new(
                T,
                MAX_BATCH_SIZE,
                OP_BATCH_PROC_PERIOD,
                OP_BATCH_BUF_CAPACITY,
            );
            while p < N {
                for (node_id, order) in orders.iter().enumerate() {
                    let mut batch = OperationIds::default();
                    for &op_id in order.iter().skip(p).take(MAX_BATCH_SIZE) {
                        batch.insert(op_id as u64);
                    }
                    on_batch_received(batch, node_id as u64, &mut protocol);
                }
                p += MAX_BATCH_SIZE;
            }
        })
    });
}

/// Measure when we keep receiving without locks
pub fn more_cpu_simple_receive(c: &mut Criterion) {
    use bench_sandbox::more_cpu::*;
    const MAX_BATCH_SIZE: usize = 100;
    const T: usize = 25; // Numer of nodes
    const N: usize = 10_000; // Number of operations

    if N % MAX_BATCH_SIZE != 0 {
        panic!("For the test N should be a multiple of MAX_BATCH_SIZE");
    }

    let mut orders = Box::new([[0; N + 1]; T + 1]); // Order of operations for each node
    let mut operations = Box::new([0usize; N + 1]); // Table of operations
    for n in 0..N {
        for order in orders.iter_mut() {
            order[n] = n; // init an order to shuffle later
        }
        operations[n] = rand::random();
    }
    let mut thread_rng = rand::thread_rng();
    for order in orders.iter_mut() {
        order.shuffle(&mut thread_rng);
    }

    c.bench_function("More cpu on receive batch", |b| {
        b.iter(|| {
            let mut p = 0;
            let mut protocol = FakeProtocol::new(T, MAX_BATCH_SIZE);
            while p < N {
                for (node_id, order) in orders.iter().enumerate() {
                    let mut batch = OperationIds::default();
                    for &op_id in order.iter().skip(p).take(MAX_BATCH_SIZE) {
                        batch.insert(op_id as u64);
                    }
                    on_batch_received(batch, node_id as u64, &mut protocol);
                }
                p += MAX_BATCH_SIZE;
            }
        })
    });
}
