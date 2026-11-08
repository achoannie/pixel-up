use std::{fs::File, io::BufReader, result};

use image::{AnimationDecoder, ImageBuffer, Rgba, RgbaImage, codecs::gif::GifDecoder, imageops};

const MARGIN: i64 = 2;
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
fn bbox(img: &RgbaImage) -> Option<(i64, i64, i64, i64)> {
    let mut found = false;
    let (mut x0, mut y0) = (i64::MAX, i64::MAX);
    let (mut x1, mut y1) = (i64::MIN, i64::MIN);

    for (x, y , pixel) in img.enumerate_pixels() {
        if pixel[3] > 0 {
            found = true;
            let (x, y) = (x as i64, y as i64);
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x + 1);
            y1 = y1.max(y + 1);
        }
    }
    if found {Some((x0, y0, x1, y1))} else {None}
}
fn crop_box(frames: &[Frame], size: Option<u32>) -> Option<(i64, i64, u32)> {
    let boxes: Vec<_> = frames.iter().filter_map(|f| bbox(&f.img)).collect();
    if boxes.is_empty() {return None;}
    let x0 = boxes.iter().map(|f| f.0).min().unwrap();
    let y0 = boxes.iter().map(|f| f.1).min().unwrap();
    let x1 = boxes.iter().map(|f| f.2).max().unwrap();
    let y1 = boxes.iter().map(|f| f.3).max().unwrap();
    let size: i64 = match size {
        Some(0) => return None,
        Some(s) => s as i64, 
        None => {
            let need = (x1-x0).max(y1-y0) + MARGIN * 2;
            (need + 7) / 8 * 8
        }
    };
    let centerx = (x0 + x1) /2; 
    let left = centerx - size/2;
    let bottom = y1 + MARGIN;
    Some((left, bottom-size, size as u32))
}
fn crop_window(img: &RgbaImage, left: i64, top: i64, size: u32) -> RgbaImage {
    let mut output = ImageBuffer::new(size, size);
    imageops::overlay(&mut output, img, -left, -top);
    output
}

fn merge_holds(frames: Vec<Frame>) -> Vec<Frame> {
    let mut result: Vec<Frame> = Vec::new();
    for frame in frames{
        // check if frame is empty, if it is, ignore and add its ms to the previous one
        let is_empty = bbox(&frame.img).is_none();
        if result.is_empty() {
            if !is_empty{result.push(frame);}
            continue; //skip the rest of the loop and go to the next frame
        }

        let last_frame= result.len()-1;

        if is_empty {
            println!("skipped an empty frame");
            // result's last frame inherits the empty frame's duration 
            result[last_frame].ms += frame.ms;
        } else if frame.img == result[last_frame].img  { // duplicated frames get merged
            result[last_frame].ms += frame.ms;
        } else { // non empty frame get pushed to result
            result.push(frame);
        }
    }
    result
}
fn main() {

    let args : Vec<String> = std::env::args().collect();
    let (src, dst) = (&args[1], &args[2]);
    let scale: u32 = args.get(3).map_or(8, |s| s.parse().expect("scale must be a number"));
    let size: Option<u32> = args.get(4).map(|s| s.parse().expect("size must be a number"));
    let mut frames = load_frames(src);
    
    println!("----checking arguments----");
    if args.len() < 3 {
        eprintln!("Required order: pixel-up <input.gif> <output.gif> [scale] [size]");
        std::process::exit(1)
    }
    println!("{src} -> {dst}, {scale}, {size:?}" );

    println!("----frame loading----");
    println!("the gif has {} frames", frames.len());
    for frame in &frames {
        println!("{}x{}, {} ms", frame.img.width(), frame.img.height(), frame.ms);
    }

    println!("----bounding box----");
    for (i, f) in frames.iter().enumerate() {
        println!("frame {i}: {:?}", bbox(&f.img));
    }

    println!("----cropping box calculation----");
    println!("crop box: {:?}", crop_box(&frames, size));


    println!("----cropping----");
    
    if let Some((left, top, size)) = crop_box(&frames, size) {
        for f in &mut frames {
            f.img = crop_window(&f.img, left, top, size);
        }
    }
    for f in &mut frames {
        let (w, h) = f.img.dimensions();
        f.img = imageops::resize(&f.img, w * scale , h * scale, imageops::FilterType::Nearest);
    }

    if dst.to_lowercase().ends_with(".png") {
        frames[0].img.save(dst).expect("can't save PNG");
        println!("{dst}: {}x{}", frames[0].img.width(), frames[0].img.height())
    }
    
}
// run `cargo test`
#[test]
fn merge_holds_total_duration_for_duplicated_frames() {
    // starting with smol dimension
    let mut img = RgbaImage::new(4, 4);
    img.put_pixel(1, 1, Rgba([255, 0, 0, 255])); //opaq red

    let frames = vec![
        Frame { img: img.clone(), ms: 100},
        Frame { img: img.clone(), ms: 100},
        Frame { img: img.clone(), ms: 100},
    ];
    let merged = merge_holds(frames);

    assert_eq!(merged.len(), 1); // 3 identical frames merged into 1
    assert_eq!(merged[0].ms, 300); // total of 3 frames 
}

#[test]
fn merge_holds_empty_last_frame() {
    // starting with smol dimension
    let mut img = RgbaImage::new(4, 4);
    img.put_pixel(1, 1, Rgba([255, 0, 0, 255])); //opaq red
    let empty_frame = RgbaImage::new(4, 4);
    let frames = vec![
        Frame { img: img.clone(), ms: 100},
        Frame { img: img.clone(), ms: 100},
        Frame { img: empty_frame, ms: 100},
    ];
    let merged = merge_holds(frames);

    assert_eq!(merged.len(), 1); // 2 identical frames + 1 empty frame -> merged into 1
    assert_eq!(merged[0].ms, 300); // total of 3 frames 
}