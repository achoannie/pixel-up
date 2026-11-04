# What I learned building pixel-up

## Types I used and why

| Type | What it is | Why I used it here |
| --- | --- | --- |
| `u8` | whole number 0–255 | one colour channel, a palette index |
| `u16` | 0–65,535 | GIF width, height and delay (the format stores them this way) |
| `u32` | 0 to ~4 billion, never negative | image width, height, pixel x/y |
| `i64` | can be negative | crop box coordinates, which can go off the canvas |
| `[u8; 3]` | fixed array of exactly 3 | one RGB colour |
| `Vec<T>` | growable list | frames, palette, pixel indices |
| `String` / `&str` | owned / borrowed text | file paths |

## Concepts
### How this project started
So as I was working on my pomodoro mobile application project which is using animals pixel gif animation to run during idle time and many more moods. I ran into a problem when the drawing software I was using (Clip Paint Studio) isn't very good at adjusting desired sizes with proper resolution after exporting. I found it would be more fitting to run a script to do that for me. I firstly came up with Python script then a Rust Project interfered that and here I am learning Rust during my autumn break.

## Brief idea
- Order of CLI: `pixel-up <input.gif> <output.gif> [scale] [size|null]`
- Gif reader: read every frame of gif in rbga images, duration in ms
- Frame loading:
    - in order to load the frames, first u need to read the file, drop a message if unable to open, decode them into individual frames, and collect them into a list
    - Why BufReader is here because it reads file in big chunk at once and not byte by byte, then keeps it in memory -> better speed. Use BufReader when you read a file in many small pieces
    - the part where decode into frames is tricky as: calling each frames f, map takes in a list of library frames and turn it into another list of frames. The .collect() gathers it into `Vec<Frame>`
        - `f` is the image crate’s frame (`image::Frame`). It holds an image and a delay, in the library’s own format.
        - `Frame { img, ms }` at the end is your struct from the top of the file.
        - `.numer_denom_ms()` turns that Delay into milliseconds, as a fraction: a numerator and a denominator. A delay of 100 ms comes back as (100, 1), meaning 100 ÷ 1.
        - in cases where den == 0 -> would crash so set by default = 120ms
    example for command line: `cargo run -- /Users/pqa/Desktop/michi.gif michi_test.gif`

### Ownership and borrowing
- In my words:
- Where it shows up in pixel-up:
- Error I hit:

### Option
- ...

## Mistakes and how I fixed them
- 2026-11-0X: forgot `mut` on `frames` → E0596. Fix: `let mut frames`.

## Python vs Rust
- Python's `getbbox()` → I had to write `bbox()` myself
- `-(-n // 8) * 8` → `(n + 7) / 8 * 8`