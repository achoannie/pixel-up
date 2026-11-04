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
### Frame loading:
- in order to load the frames, first u need to read the file, drop a message if unable to open, decode them into individual frames, and collect them into a list
- Why BufReader is here because it reads file in big chunk at once and not byte by byte, then keeps it in memory -> better speed. Use BufReader when you read a file in many small pieces
- the part where decode into frames is tricky as: calling each frames f, map takes in a list of library frames and turn it into another list of frames. The .collect() gathers it into `Vec<Frame>`
        - `f` is the image crate’s frame (`image::Frame`). It holds an image and a delay, in the library’s own format.
        - `Frame { img, ms }` at the end is your struct from the top of the file.
        - num / den turns it into whole milliseconds.
        - `.numer_denom_ms()` turns that Delay into milliseconds, as a fraction: a numerator and a denominator. A delay of 100 ms comes back as (100, 1), meaning 100 ÷ 1.
        - in cases where den == 0 -> would crash so set by default = 120ms
    example for command line: `cargo run -- /Users/pqa/Desktop/michi.gif michi_test.gif`
### Bounding box
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


## Mistakes and how I fixed them
- 2026-11-04: wrote `Option<i64, i64, i64, i64>` → E0107. `Option` holds
  ONE thing, so the 4 numbers need to be a tuple: `Option<(i64, i64, i64, i64)>` 
- 2026-11-04: wrote `x1.max(x) + 1` instead of `x1.max(x + 1)`. No compiler
  error, but the +1 piled up for every visible pixel (got 7 instead of 4).
  Lesson: the compiler can't catch wrong math. Test with an example where
  you already know the answer
- 2026-11-04: `for f in frames` moves the Vec; use `&frames` if I need it after
- 2026-11-03: used `print!` instead of `println!` → no newlines