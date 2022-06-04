pub mod general;
pub mod cpu;
pub mod node;
pub mod energy;
pub mod temperature;
pub mod pressure;
pub mod manager;

#[cfg(feature = "opencl3")]
pub mod gpu;