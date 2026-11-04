use std::{fs::File, io::BufReader};

use image::{AnimationDecoder, RgbaImage, codecs::gif::GifDecoder};

fn main() {
    // input output scale (size)
    let args : Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Required order: pixel-up <input.gif> <output.gif> [scale] [size]");
        std::process::exit(1)
    }
    let (src, dst) = (&args[1], &args[2]);
    let scale: u32 = args.get(3).map_or(8, |s| s.parse().expect("scale must be a number"));
    let size: Option<u32> = args.get(4).map(|s| s.parse().expect("size must be a number"));
    println!("{src} -> {dst}, {scale}, {size:?}" );

    struct Frame {
        img: RgbaImage,
        ms: u32
    }
    fn load_frames(path: &str) -> Vec<Frame> {
        let file = File::open(path).expect("cannot open the file");
        let file_reader = BufReader::new(file);
        let decoder = GifDecoder::new(file_reader).expect("Not a gif");
        decoder.into_frames().collect_frames().expect("bad gif frames").into_iter().map(|f| {
            let (num, den) = f.delay().numer_denom_ms();
            let ms = if den==0 {120} else {
                num/den
            };
            Frame { img: f.into_buffer(), ms }
        }).collect()
    }
    let frames = load_frames(src);
    println!("the gif has {} frames", frames.len());
    for frame in &frames {
        println!("{}x{}, {} ms", frame.img.width(), frame.img.height(), frame.ms);
    }
    
    fn bbox(img: &RgbaImage) -> Option<(i64, i64, i64, i64)> {
        let mut found = false;
        let (mut x0, mut y0) = (i64::MAX, i64::MAX);
        let (mut x1, mut y1) = (i64::MIN, i64::MIN);

        for (x, y , pixel) in img.enumerate_pixels() {
            // see if alpha in rgba is transparent (0) or is opaque (1)
            if pixel[3] > 0 {
                found = true;
                let (x, y) = (x as i64, y as i64);
                // compare between 2 value which is smaller
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x) + 1;
                y1 = y1.max(y) + 1;
            }
        }
        if found {Some((x0, y0, x1, y1))} else {None}
    }
    for (i, f) in frames.iter().enumerate() {
        println!("frame {i}: {:?}", bbox(&f.img));
    }
}
