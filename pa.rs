use std::libc::{c_int, c_void, size_t};
use std::ptr::null;
use std::cast;
use std::vec;
use std::comm;
use std::task;

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
			let s: *pa_simple = pa_simple_new(null(), "name".as_c_str(|x| x), 2, null(), "stream_name".as_c_str(|x| x), &ss, null(), null(), &error);
			assert_eq!(error, 0);
			loop {
				let mut buffer: ~[i16] = ~[0, ..256];
				pa_simple_read(s, cast::transmute(vec::raw::to_mut_ptr(buffer)), 512, &error);
				assert_eq!(error, 0);
				let f32Buffer: ~[f32] = buffer.iter().transform(|&i| (i as f32)).collect();
				cData.send(f32Buffer);
			}
		}
	}
	return pData
}
