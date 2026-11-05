## How the program works (so far)

### 1. Reading the command-line arguments
Order: `pixel-up <input.gif> <output.gif> [scale] [size]`

- `std::env::args().collect()` gives all arguments as a `Vec<String>`
- `args[0]` is the program itself, so my first real argument is `args[1]`
- fewer than 3 items → print the usage with `eprintln!` and `exit(1)`
  (1 = "something went wrong"; 0 = success)
- `args[3]` would crash if it doesn't exist → `args.get(3)` returns an `Option` instead
- `scale`: `.map_or(8, ...)` → use the number if given, otherwise 8
- `size`: `.map(...)` → stays an `Option<u32>`, because "no size" means auto-crop (a behaviour, not a number)
- arguments are always text → `.parse()` turns `"4"` into `4`; the type annotation (`u32`) tells it which number type

### 2. The `Frame` struct
```rust
struct Frame {
    img: RgbaImage, // the pixels
    ms: u32,        // how long the frame shows
}
```

### Frame loading:
- In order to load the frames, first u need to read the file, drop a message if unable to open, decode them into individual frames, and collect them into a list
- Why `BufReader` is here because it reads file in big chunk at once and not byte by byte, then keeps it in memory -> better speed. Use `BufReader` when you read a file in many small pieces
- `into_frames()` only exists on buffered readers, so it won't compile without it
- `GifDecoder` instead of `ImageReader::open().decode()`: `.decode()` only gives the
  first frame, but I need every frame + its delay
- the part where decode into frames is tricky as: calling each frames f, map takes in a list of library frames and turn it into another list of frames. The .collect() gathers it into `Vec<Frame>`
    - `f` is the image crate’s frame (`image::Frame`). It holds an image and a delay, in the library’s own format.
    - `.map(|f| ...)` turns every library frame (`image::Frame`) into my own `Frame`
    - `f.delay().numer_denom_ms()` → the delay as a fraction (num = numerator, den = denominator),
    e.g. `(100, 1)` → `num / den` = 100 ms
    - in cases where den == 0 -> would crash (division by zero) so set by default = 120ms

### Bounding box (`bbox`)
Bounding box is the rectangle bordering around every non-transparent pixel. 

Side note: Pixel perfectly fits this as for pixel, its either a colored pixel or not regardless of opacity layers => opaque (alpha = 1) and transparent (alpha = 0) in rgba

Example of how bounding box is calculated: imagine a 6x5 image. `#` is a visible pixel.
```
      x→  0 1 2 3 4 5
y=0       . . . . . .
y=1       . . # . . .
y=2       . # # # . .
y=3       . . # . . .
y=4       . . . . . .
```
Looking at the `#`s:

- the leftmost one is at x = 1
- the rightmost one is at x = 3
- the top one is at y = 1
- the bottom one is at y = 3

So the box goes from (1, 1) to (3, 3). That’s the whole idea: the smallest x, the smallest y, the largest x and the largest y among the visible pixels.

Concepts: `img.enumerate_pixels()` yields `(x, y, pixel)` for every pixel.

### Crop box (`cropbox`)
One square box that fits the drawing for **every** frame, in case during animation the pixel jump around and get cropped away
`size` has 3 cases:
- `None` -> auto
- `Some(0)` -> don't crop (return None)
- `Some(s)` -> user's set `s`. example: `Some(32)` -> 32x32

Steps:
1. `bbox` every frame; `filter_map` drops the empty ones (`None`)
2. all frames empty → `None`, nothing to crop around
3. combine all boxes: smallest left/top, biggest right/bottom
   (`.map(|b| b.0).min()`: take the first number of each box, find the smallest)
   `.unwrap()` is safe because I checked `is_empty()` first
4. auto size = the bigger of width/height + 2px margin on both sides, rounded up to a multiple of 8
5. center it horizontally: `center_x = (x0 + x1) / 2`, `left = center_x - size / 2`
6. bottom = 2px below the feet (`y1 + MARGIN`), top = `bottom - size`
7. return `Some((left, top, size))`

**Why `size` and `MARGIN` are `i64`:** not because they can be negative, but because they do
math with positions that can be (`left` goes negative when the drawing is near the edge).
Rust doesn't let you mix number types.

**Rounding up trick:** `(need + 7) / 8 * 8`. Integer division rounds down, and adding 7 first
pushes it to the next multiple. On unsigned types, Rust has `n.next_multiple_of(8)` built in.

### Crop, enlarge, save a PNG file
Apply the crop to every frame from `cropbox` calculation, scale it up with nearest neighbor mode (most fitted for pixel mode), and save the first frame as a PNG so you can see in demo how it works.

> tiny problem: Run a same dst file name will overwrite it with a new file. Solution: Should make a eprintl to warn or a small feature later

sources: 

imageops modules:
- [docs.rs: resize](https://docs.rs/image/latest/image/imageops/index.html)
- [docs.rs: overlay](docs.rs/image/latest/image/imageops/fn.overlay.html)
- [from-pixel from ImageBuffer: Constructs a new `ImageBuffer` by copying a pixel
](https://docs.rs/image/latest/image/struct.ImageBuffer.html#method.from_pixel)



## Mistakes and how I fixed them
- 2026-11-04: wrote `Option<i64, i64, i64, i64>` → E0107. `Option` holds
  ONE thing, so the 4 numbers need to be a tuple: `Option<(i64, i64, i64, i64)>` 
- 2026-11-04: wrote `x1.max(x) + 1` instead of `x1.max(x + 1)`. No compiler
  error, but the +1 piled up for every visible pixel (got 7 instead of 4).
  Lesson: the compiler can't catch wrong math. Test with an example where
  you already know the answer
- 2026-11-04: `for f in frames` moves the Vec; use `&frames` if I need it after
- 2026-11-03: used `print!` instead of `println!` → no newlines
- 2026-11-05: michi (tester) came out with bad quality: blurry stripes between colours. Turn out it wasn't my code but because it was the demo gif my Clip Studio first ever exported which had some disported pixels. Quite interesting
- 2026-11-05: `RgbaImage([0,0,0,0])` → E0423. `Rgba` is ONE pixel,
  `RgbaImage` is a whole image. Needed `Rgba([0, 0, 0, 0])` (and the import)
- 2026-11-05: put `struct`, `fn` and `const` inside `main`. It works, but future testing it can't see them. Definitions go at the top level; `let` stay
  inside functions