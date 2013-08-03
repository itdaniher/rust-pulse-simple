extern mod sdl;
extern mod extra;

use extra::time;
use std::uint;
use std::comm;

pub fn drawVectorAsBarPlot (screen: &sdl::video::Surface, data: ~[f32]){
	// black screen background
	screen.fill_rect(Some(sdl::Rect {x: 0 as i16, y: 0 as i16, w: screen.get_width(), h: screen.get_height()}), sdl::video::RGB(0,0,0));
	// calculate bar width
	let width: f32 = screen.get_width() as f32 / (data.len() as f32);
	let height: f32 = screen.get_height() as f32;
	// find max value
	let &dmax: &f32 = data.iter().max().get();
	// calculate height scale value
	let scale: f32 = height / dmax;
	assert!(width > 1.0);
	for uint::range(0, data.len()) | i | {
		let r = sdl::Rect {
			x: ((screen.get_width() as f32)- width*(i as f32 + 1.0)) as i16,
			y: (height - scale*data[i]) as i16,
			w: (width) as u16,
			h: (scale*data[i]) as u16};
		screen.fill_rect(Some(r), sdl::video::RGB(0,127,0)); //+(rand::rng().gen::<u8>())/2,0));
	}
}


pub fn spawnVectorVisualSink() -> (comm::Port<sdl::event::Key>, comm::Chan<~[f32]>){
	let (pData, cData): (comm::Port<~[f32]>, comm::Chan<~[f32]>) = comm::stream();
	let (pUser, cUser): (comm::Port<sdl::event::Key>, comm::Chan<sdl::event::Key>) = comm::stream();
	do sdl::start {
		let mut lastDraw: u64 = 0;
		sdl::init([sdl::InitVideo]);
		sdl::wm::set_caption("rust-sdl", "rust-sdl");
		let screen = match sdl::video::set_video_mode(800, 600, 32, [sdl::video::HWSurface],
			                                                        [sdl::video::DoubleBuf]) {
			Ok(screen) => screen,
			Err(err) => fail!(fmt!("failed to set video mode: %s", err))
		};
		'main : loop {
			if pData.peek() {
				let d = pData.recv();
				drawVectorAsBarPlot(screen, d);
			}
			'event : loop {
				let ev = sdl::event::poll_event();
				match ev {
					sdl::event::QuitEvent => break 'main,
					sdl::event::NoEvent => {break 'event},
					sdl::event::KeyEvent (a,b,c,d) => {if (b == true) {cUser.send(a)}},
					_ => {println(fmt!("%?", ev));}
				}
			}
			if ((time::precise_time_ns() - lastDraw) > (1f32/30f32*1e9 as u64)) {
				lastDraw = time::precise_time_ns();
				screen.flip();
			}
		}
		sdl::quit();
	}
	return (pUser, cData);
}
