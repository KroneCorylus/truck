mod boolean;
mod fixtures;
mod local;
mod modeling;
mod projection;
mod step;
mod tessellation;

#[cfg(feature = "allocations")]
#[global_allocator]
static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

fn main() { divan::main(); }
