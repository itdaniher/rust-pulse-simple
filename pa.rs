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

#[link_args = "-lpulse -lpulse-simple"]
extern {
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
	) -> *pa_simple;
	fn pa_simple_read(s: *pa_simple, data: *mut c_void, bytes: size_t, error: *c_int) -> c_int;
	fn pa_simple_write(s: *pa_simple, data: *c_void, bytes: size_t, error: *c_int) -> c_int;
	// pa_simple_flush(pa_simple *s, int *error) -> c_int;
	fn pa_simple_get_latency(s: *pa_simple, error: *c_int) -> u64;
}

pub fn buildPASourceBlock() -> comm::Port<~[f32]> {
	let (pData, cData): (comm::Port<~[f32]>, comm::Chan<~[f32]>) = comm::stream();
	
	let ss = pa_sample_spec { format: 3, rate: 8000, channels: 1 };
	// pa_stream_direction_t -> enum, record = 2, playback = 1
	do task::spawn_sched(task::SingleThreaded) {
		unsafe {
			let mut error: c_int = 0;
			let s: *pa_simple = pa_simple_new(null(), "rust-pa-simple".as_c_str(|x| x), 2, null(), "pa-source".as_c_str(|x| x), &ss, null(), null(), &error);
			assert_eq!(error, 0);
			'main : loop {
				let mut buffer: ~[i16] = ~[0, ..256];
				pa_simple_read(s, cast::transmute(vec::raw::to_mut_ptr(buffer)), 512, &error);
				assert_eq!(error, 0);
				let f32Buffer: ~[f32] = buffer.iter().transform(|&i| (i as f32)).collect();
				cData.send(f32Buffer);
			}
		}
	}
	return pData;
}

pub fn buildPASinkBlock() -> comm::Chan<~[f32]> {
	let (pData, cData): (comm::Port<~[f32]>, comm::Chan<~[f32]>) = comm::stream();
	let sRate: f32 = 44100.0;
	let freq: f32 = 1.0/256.0/sRate;
	// f32 @ 44.1k
	let ss = pa_sample_spec { format: 5, rate: sRate as u32, channels: 1 };
	do task::spawn_sched(task::SingleThreaded) {
		let mut error: c_int = 0;
		unsafe {
			let s: *pa_simple = pa_simple_new(null(), "rust-pa-simple".as_c_str(|x| x), 1, null(), "pa-sink".as_c_str(|x| x), &ss, null(), null(), &error);
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
	let sRate: f32 = 44100.0;
	let freq: f32 = 5.0/(256.0/sRate);
	println(fmt!("%?", freq));
	let z: ~[f32] = ~[0.0, ..256];
	let sin: ~[f32] = z.iter().enumerate().transform(|(x, &y)| {num::sin(((x as f32)/sRate)*freq*6.28319)}).collect();
	let c = buildPASinkBlock();
	println(fmt!("%?", sin));
	for 100.times {
		c.send(sin.clone());
	}
	c.send(~[]);
}
