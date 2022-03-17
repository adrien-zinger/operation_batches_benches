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
const OP_BATCH_PROC_PERIOD: u64 = 200;
const T: usize = 25; // Numer of nodes
const N: usize = 10_000; // Number of operations
                         // for this test we need to have the same buffer size as the input
                         // operations number
const OP_BATCH_BUF_CAPACITY: usize = N;

const _: () = {
    if T == 0 {
        panic!("We need a number of nodes > 0");
    }
    if N % MAX_BATCH_SIZE != 0 {
        panic!("For the test N should be a multiple of MAX_BATCH_SIZE");
    }
};

fn print_output(measures: Vec<(NodeId, OperationIds)>) {
    println!("Total batches required: {}", measures.len());
    println!(
        "We asked an average of {} times each node",
        measures.len() / T
    );
    let mut ids = OperationIds::new();
    let mut nodes = vec![0; T + 1];
    for (node_id, operation_ids) in measures.iter() {
        *nodes.get_mut(*node_id as usize).unwrap() += 1;
        ids.extend(operation_ids.clone());
    }
    println!("Asking table by nodes:\n{:?}", nodes);
    println!("Correctly processed: {}", ids.len() == N);
    assert_eq!(ids.len(), N);
}

pub fn process() {
    let protocol = Arc::new(Mutex::new(new_protocol()));
    let sig_stop = Arc::new(AtomicBool::new(true));
    let batch_sender = run_batch_sender(protocol.clone());
    let batch_receiver = run_operations_asked_receiver(protocol.clone());
    let operation_asker = run_operation_asker(protocol.clone(), sig_stop.clone());
    let send_loop = run_send_loop(protocol, sig_stop.clone());
    batch_sender.join().unwrap();
    let operations_asked = batch_receiver.join().unwrap();
    sig_stop.store(false, Ordering::Relaxed);
    operation_asker.join().unwrap();
    send_loop.join().unwrap();
    print_output(operations_asked);
}

fn new_protocol() -> FakeProtocol {
    FakeProtocol::new(
        T,
        MAX_BATCH_SIZE,
        OP_BATCH_PROC_PERIOD,
        OP_BATCH_BUF_CAPACITY,
    )
}

fn run_batch_sender(protocol: Arc<Mutex<FakeProtocol>>) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let mut p = 0;
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
        while p < N {
            for (node_id, order) in orders.iter().enumerate() {
                let mut batch = OperationIds::default();
                for &op_id in order.iter().skip(p).take(MAX_BATCH_SIZE) {
                    batch.insert(op_id as u64);
                }
                on_batch_received(batch, node_id as u64, &mut protocol.lock().unwrap());
            }
            p += MAX_BATCH_SIZE;
        }
    })
}

/// Ici nous avons la simulation de nos demande d'operations,
/// ces demandes sont fait suite a des receptions de batch de la part des
/// autres noeuds.
///
/// On reserve en cache la liste des demandes de notre noeuds pour
/// l'analyser en valeur de retour à la fin du process de mesure.
///
///
/// Observation suite aux mesures:
/// On remarque que plus on va mettre du temps à recevoir des operations,
/// plus on redemandera de batch mais pas toujours aux mêmes.
fn run_operations_asked_receiver(
    protocol: Arc<Mutex<FakeProtocol>>,
) -> JoinHandle<Vec<(NodeId, OperationIds)>> {
    const MIN_SLEEP: u64 = 300;
    const MAX_SLEEP: u64 = 600;
    std::thread::spawn(move || {
        let mut cache = vec![];
        let mut diff_op = OperationIds::new();
        let mut thread_rng = rand::thread_rng();
        while diff_op.len() < N {
            // little sleep that simulate the time of a communication
            // for the example it's not needed to be very big
            let rand_sleep = Duration::from_nanos(thread_rng.gen_range(MIN_SLEEP..MAX_SLEEP));
            std::thread::sleep(rand_sleep);
            let opt_asked = ASK_BATCH_QUEUE.lock().unwrap().pop_front();
            if let Some((node_id, operation_ids)) = opt_asked {
                cache.push((node_id, operation_ids.clone()));
                diff_op.extend(operation_ids.clone());
                let mut operations = OperationMap::new();
                for id in operation_ids {
                    operations.insert(id, String::new());
                }
                on_operation_received(node_id, operations, &mut protocol.lock().unwrap());
            }
        }
        cache
    })
}

/// Ici je souhaite simuler la boucle qui process `on_send_loop`.
/// On va aller bcp plus vite que dans la réalité mais on va essayer de rendre
/// ça un peu plus lent que le reste quand même.
///
/// On ne le lance que pour notre noeud local. Cette loop simulée pour les
/// autres dans [run_operation_asker]. En effet, a leur facon il redemande à
/// chaque tour de boucle a notre noeud local des operations.
fn run_send_loop(protocol: Arc<Mutex<FakeProtocol>>, stop: Arc<AtomicBool>) -> JoinHandle<()> {
    const MIN_SLEEP: u64 = 50;
    const MAX_SLEEP: u64 = 200;
    std::thread::spawn(move || {
        let mut thread_rng = rand::thread_rng();
        while stop.load(Ordering::Relaxed) {
            let rand_sleep = Duration::from_nanos(thread_rng.gen_range(MIN_SLEEP..MAX_SLEEP));
            std::thread::sleep(rand_sleep);
            on_send_loop(&mut protocol.lock().unwrap())
        }
    })
}

/// Le noeud envoie des batches et c'est dans ce thread qu'onva gere ce que
/// les noeuds demande en retour.
fn run_operation_asker(
    protocol: Arc<Mutex<FakeProtocol>>,
    stop: Arc<AtomicBool>,
) -> JoinHandle<Vec<(NodeId, OperationIds)>> {
    const MIN_SLEEP: u64 = 2;
    const MAX_SLEEP: u64 = 20;
    std::thread::spawn(move || {
        let mut thread_rng = rand::thread_rng();
        let mut cache = vec![];
        let mut protocols = (0..N)
            .map(|_| {
                let mut protocol = new_protocol();
                protocol.is_measured = false;
                protocol
            })
            .collect::<Vec<FakeProtocol>>();
        while stop.load(Ordering::Relaxed) {
            let rand_sleep = Duration::from_nanos(thread_rng.gen_range(MIN_SLEEP..MAX_SLEEP));
            std::thread::sleep(rand_sleep);
            let opt_node_batch = {
                let mut guard = BATCH_SEND_QUEUE.lock().unwrap();
                guard.shuffle(&mut thread_rng);
                guard.pop()
            };
            if let Some((node_id, operation_ids)) = opt_node_batch {
                cache.push((node_id, operation_ids.clone()));
                let ask_set = on_batch_received(
                    operation_ids,
                    0, /* not needed because it's a mock */
                    &mut protocols[node_id as usize],
                );
                on_ask_received(node_id, ask_set, &mut protocol.lock().unwrap());
            }
        }
        cache
    })
}
