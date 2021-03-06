extern mod extra;
extern mod kissfft;
extern mod dsputils;
extern mod videoSinkSDL1;
extern mod pa;

fn main() {
	let (p1, c1) = kissfft::buildFFTBlock(256*8, true);
	let (p2, c2) = videoSinkSDL1::spawnVectorVisualSink();
	let pi = pa::buildPASourceBlock(44100, 256*8);
	loop {
		c1.send(dsputils::asRe(pi.recv()));
		let x: ~[f32] = dsputils::asF32(p1.recv());
		let y: ~[f32] = x.iter().map(|&x| x.abs()).collect();
		c2.send(y.slice(0, 256*2).to_owned());
	}
}
