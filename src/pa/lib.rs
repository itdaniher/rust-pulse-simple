use std::libc::{c_int, c_void, size_t};
use std::ptr::null;
use std::cast;
use std::vec;
use std::comm;
use std::task;
use std::num;


// opaque struct
struct pa_simple;

struct pa_sample_spec {
	format: c_int,
	rate: u32,
	channels: u8
}

#[link_args = "-lpulse -lpulse-simple"] extern {}

externfn!(
	fn pa_simple_new(
		server: *c_void,
		name: *i8,
		dir: c_int,
		dev: *c_void,
		stream_name: *i8,
		ss: *pa_sample_spec,
		pa_channel_map: *c_void,
		pa_buffer_attr: *c_void,
		error: *c_int
	) -> *pa_simple
)
externfn!(
	fn pa_simple_read(s: *pa_simple, data: *mut c_void, bytes: size_t, error: *c_int) -> c_int)
externfn!(
	fn pa_simple_write(s: *pa_simple, data: *c_void, bytes: size_t, error: *c_int) -> c_int)
externfn!(
	fn pa_simple_flush(s: *pa_simple, error: *c_int) -> c_int)
externfn!(
	fn pa_simple_get_latency(s: *pa_simple, error: *c_int) -> u64)


pub fn buildPASourceBlock(sRate: uint, bSize: uint) -> comm::Port<~[f32]> {
	let (pData, cData): (comm::Port<~[f32]>, comm::Chan<~[f32]>) = comm::stream();
	
	let ss = pa_sample_spec { format: 3, rate: sRate as u32, channels: 1 };
	// pa_stream_direction_t -> enum, record = 2, playback = 1
	do task::spawn_sched(task::SingleThreaded) {
		unsafe {
			let error: c_int = 0;
			let s: *pa_simple = pa_simple_new(null(), "rust-pa-simple-source".to_c_str().unwrap(), 2, null(), "pa-source".to_c_str().unwrap(), &ss, null(), null(), &error);
			assert_eq!(error, 0);
			'main : loop {
				let mut buffer: ~[i16] = vec::from_elem(bSize, 0i16);
				pa_simple_read(s, cast::transmute(vec::raw::to_mut_ptr(buffer)), (bSize*2) as u64, &error);
				assert_eq!(error, 0);
				let f32Buffer: ~[f32] = buffer.iter().map(|&i| (i as f32)).collect();
				cData.send(f32Buffer);
			}
		}
	}
	return pData;
}

pub fn buildPASinkBlock(sRate: uint) -> comm::Chan<~[f32]> {
	let (pData, cData): (comm::Port<~[f32]>, comm::Chan<~[f32]>) = comm::stream();
	let ss = pa_sample_spec { format: 5, rate: sRate as u32, channels: 1 };
	do task::spawn_sched(task::SingleThreaded) {
		let error: c_int = 0;
		unsafe {
			let s: *pa_simple = pa_simple_new(null(), "rust-pa-simple-sink".to_c_str().unwrap(), 1, null(), "pa-sink".to_c_str().unwrap(), &ss, null(), null(), &error);
			println(fmt!("%?", pa_simple_get_latency(s, &error)));
			'main : loop {
				let samps: ~[f32] = pData.recv();
				if (samps == ~[]) { break 'main }
				let size: size_t = (samps.len() as u64)*4;
				pa_simple_write(s, cast::transmute(vec::raw::to_ptr(samps)), size, &error);
			}
		}
	}
	return cData;
}

fn main() {
	let sRate = 44100;
	let freq = 5.0/(256.0/(sRate as float));
	println(fmt!("%?", freq));
	let z = ~[0, ..256];
	let sin: ~[f32] = z.iter().enumerate().map(|(x, &y)| {num::sin(((x as float)/(sRate as float))*freq*6.28319) as f32}).collect();
	let c = buildPASinkBlock(sRate);
	println(fmt!("%?", sin));
	for x in range(0, 100) {
		c.send(sin.clone());
	}
	c.send(~[]);
}
