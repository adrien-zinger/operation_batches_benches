use std::{sync::{atomic::AtomicBool, Arc, Mutex}, collections::HashSet};
use criterion::{criterion_group, criterion_main, Criterion};
use rand::{seq::SliceRandom, Rng};
mod implem;
use implem::receive_and_ask::{less_cpu_receive_and_ask, more_cpu_receive_and_ask};
use implem::simple_receive_batch::{less_cpu_simple_receive, more_cpu_simple_receive};



// TODO same test as less_cpu_simple_receive but with locks outsides (calls of asks)

criterion_group!(benches, less_cpu_simple_receive, more_cpu_simple_receive, less_cpu_receive_and_ask, more_cpu_receive_and_ask);
criterion_main!(benches);
