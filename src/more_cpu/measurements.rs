use super::*;
use rand::{seq::SliceRandom, Rng};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::JoinHandle,
    time::Duration,
};

const MAX_BATCH_SIZE: usize = 100;
const T: usize = 25; // Numer of nodes
const N: usize = 10_000; // Number of operations
                         // for this test we need to have the same buffer size as the input
                         // operations number

const _: () = {
    if T == 0 {
        panic!("We need a number of nodes > 0");
    }
    if N % MAX_BATCH_SIZE != 0 {
        panic!("For the test N should be a multiple of MAX_BATCH_SIZE");
    }
};

fn print_output(measures: Vec<WantOperations>) {
    let mut ids = OperationIds::new();
    let mut nodes = vec![0; T + 1];
    for wanted in measures.iter() {
        for (node_id, operation_ids) in wanted.iter() {
            *nodes.get_mut(*node_id as usize).unwrap() += 1;
            ids.extend(operation_ids.clone());
        }
    }
    println!("Asking table by nodes:\n{:?}", nodes);
    println!("Correctly processed: {}", ids.len() == N);
    assert_eq!(ids.len(), N);
}

pub fn process() {
    let protocol = Arc::new(Mutex::new(new_protocol()));
    let sig_stop = Arc::new(AtomicBool::new(true));
    let batch_sender = run_batch_sender(protocol.clone());
    let asking_loop = run_asking_loop(protocol.clone(), sig_stop.clone());
    let op_sender = run_operations_asked_receiver(protocol);
    batch_sender.join().unwrap();
    sig_stop.store(false, Ordering::Relaxed);
    let operations_asked = asking_loop.join().unwrap();
    op_sender.join().unwrap();
    print_output(operations_asked);
}

fn new_protocol() -> FakeProtocol {
    FakeProtocol::new(T, MAX_BATCH_SIZE)
}

fn run_batch_sender(protocol: Arc<Mutex<FakeProtocol>>) -> JoinHandle<()> {
    //const MIN_SLEEP: u64 = 30;
    //const MAX_SLEEP: u64 = 60;
    std::thread::spawn(move || {
        // Chaque noeud va envoyer les mêmes operations mais dans un ordre different
        // N batches de MAX_BATCH_SIZE operations
        // Reminder: on a T noeuds
        let mut orders = Box::new([[0; N + 1]; T + 1]); // Order of operations for each node

        // On conserve les operations ici
        let mut operations = Box::new([0usize; N + 1]); // Table of operations
        for n in 0..N {
            for order in orders.iter_mut() {
                order[n] = n; // init an order to shuffle later
            }
            operations[n] = rand::random(); // Generation de l'identifiant de l'operation
        }
        let mut thread_rng = rand::thread_rng();

        // Voilà, ici chaque noeud definis dans quel ordre il va envoyer les opérations
        // ... Puisque c'est possible et que c'est probablement le pire
        // cas, on randomize l'ordre d'envoie des operations. On créé le batch après -->*ici
        for order in orders.iter_mut() {
            order.shuffle(&mut thread_rng);
        }
        let mut p = 0;

        while p < N {
            for (node_id, order) in orders.iter().enumerate() {
                //let rand_sleep = Duration::from_nanos(thread_rng.gen_range(MIN_SLEEP..MAX_SLEEP));
                //std::thread::sleep(rand_sleep);
                let mut batch = OperationIds::default(); // *ici <--
                for &op_id in order.iter().skip(p).take(MAX_BATCH_SIZE) {
                    batch.insert(op_id as u64);
                }
                let mut guard = protocol.lock().unwrap();
                on_batch_received(batch, node_id as u64, &mut guard);
            }
            // tant qu'on a pas envoyé N operations, on continue
            p += MAX_BATCH_SIZE;
        }
    })
}

/// Ici on s'occupe de lancer et relancer la boucle de demande.
/// On va aller beaucoup plus vite que dans les cas reelles mais
/// bon... les noeuds distants aussi iront très très vite!
///
/// On guard en cash ce qu'on demande
fn run_asking_loop(
    protocol: Arc<Mutex<FakeProtocol>>,
    stop: Arc<AtomicBool>,
) -> JoinHandle<Vec<WantOperations>> {
    const MIN_SLEEP: u64 = 20;
    const MAX_SLEEP: u64 = 100;
    std::thread::spawn(move || {
        let mut thread_rng = rand::thread_rng();
        let mut cache = vec![];
        while stop.load(Ordering::Relaxed) {
            let rand_sleep = Duration::from_nanos(thread_rng.gen_range(MIN_SLEEP..MAX_SLEEP));
            std::thread::sleep(rand_sleep);
            let mut guard = protocol.lock().unwrap();
            on_asking_loop(&mut guard);
            cache.push(guard.wanted.clone());
        }
        cache
    })
}

fn run_operations_asked_receiver(
    protocol: Arc<Mutex<FakeProtocol>>,
) -> JoinHandle<Vec<(NodeId, OperationIds)>> {
    const MIN_SLEEP: u64 = 1;
    const MAX_SLEEP: u64 = 2;
    std::thread::spawn(move || {
        let mut cache = vec![];
        let mut diff_op = OperationIds::new();
        let mut thread_rng = rand::thread_rng();
        while diff_op.len() < N {
            let rand_sleep = Duration::from_millis(thread_rng.gen_range(MIN_SLEEP..=MAX_SLEEP));
            std::thread::sleep(rand_sleep);
            let mut batches = ASK_BATCH_QUEUE.lock().unwrap();
            batches.shuffle(&mut thread_rng);
            let opt_asked = batches.pop();
            std::mem::drop(batches);
            if let Some((node_id, operation_ids)) = opt_asked {
                cache.push((node_id, operation_ids.clone()));
                diff_op.extend(operation_ids.clone());
                let mut operations = AskedOperations::new();
                for id in operation_ids {
                    operations.insert(id, Some(String::new()));
                }
                on_operation_received(node_id, operations, &mut protocol.lock().unwrap());
            }
        }
        cache
    })
}
