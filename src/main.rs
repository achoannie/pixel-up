use image::RgbaImage;

fn main() {
    struct Frame {
        img: RgbaImage,
        ms: u32
    }
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

}
