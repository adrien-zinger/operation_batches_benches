mod less_cpu;
mod more_cpu;

fn main() {
    less_cpu::measurements::process();
    more_cpu::measurements::process();
}
