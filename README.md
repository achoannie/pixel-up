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


## Mistakes and how I fixed them
- 2026-11-04: wrote `Option<i64, i64, i64, i64>` → E0107. `Option` holds
  ONE thing, so the 4 numbers need to be a tuple: `Option<(i64, i64, i64, i64)>` 
- 2026-11-04: wrote `x1.max(x) + 1` instead of `x1.max(x + 1)`. No compiler
  error, but the +1 piled up for every visible pixel (got 7 instead of 4).
  Lesson: the compiler can't catch wrong math. Test with an example where
  you already know the answer
- 2026-11-04: `for f in frames` moves the Vec; use `&frames` if I need it after
- 2026-11-03: used `print!` instead of `println!` → no newlines