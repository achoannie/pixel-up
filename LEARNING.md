# What I learned building pixel-up (ﾉ◕ヮ◕)ﾉ*:･ﾟ✧

## How this project started (｡•̀ᴗ-)✧
I was working on my pomodoro mobile app, which uses little animal pixel-art GIF animations for idle time and different moods. I ran into a problem: my drawing software (Clip Studio Paint) isn't very good at scaling pixel art to the size I want while keeping it crisp. A script seemed like a better fit. I started with a Python script, then a Rust project took over, and here I am learning Rust during my autumn break.

## The big picture (map of the whole program) ٩(◕‿◕｡)۶

```
input.gif / input.png
  │
  ├─ load_frames ────→ list of Frames (image + ms)
  ├─ crop_box ───────→ one square for ALL frames (or None)
  ├─ crop_window ────→ every frame cut to that square
  ├─ scale_up ───────→ every pixel becomes a solid block
  │
  ├─ .png output ────→ save the first frame, done!
  │
  └─ .gif output ────→ save_gif:
        ├─ merge_holds ────→ repeated frames merged, empty frames dropped
        ├─ collect_colors ─→ every colour used, once        ┐ Step 8:
        ├─ build_lut ──────→ colour → number (starting at 1) ┘ make the key
        ├─ flat_palette ───→ the key as one row of numbers   ┐ Step 9:
        └─ to_indices ─────→ every pixel as a number         ┘ use the key        
```

A GIF needs exactly two things: **a colour key (palette)** once, and **a grid of numbers per frame**. Steps 8 and 9 only exist to make those two things.

## Types I used and why (・ω・)ﾝ?

| Type | What it is | Why I used it here |
| --- | --- | --- |
| `u8` | whole number 0–255 (one "byte") | one colour channel, a palette index |
| `u16` | 0–65,535 | GIF width, height and delay (the format stores them this way) |
| `u32` | 0 to ~4 billion, never negative | image width, height, pixel x/y, scale |
| `i64` | can be negative | crop box coordinates, which can go off the canvas |
| `[u8; 3]` | fixed array of exactly 3 | one RGB colour |
| `Vec<T>` | growable list | frames, palette, pixel indices |
| `Option<T>` | "maybe a value": `Some(x)` or `None` | `bbox` of an empty frame, the optional size |
| `String` / `&str` | owned / borrowed text | file paths |
| `HashMap<K, V>` | lookup table (like a Python dict) | colour → palette number |

## How the program works

### 1. Reading the command-line arguments (。・∀・)ノ
Order: `pixel-up <input.gif|png> <output.gif|png> [scale] [size]`

- `std::env::args().collect()` gives all arguments as a `Vec<String>`
- `args[0]` is the program itself, so my first real argument is `args[1]`
- fewer than 3 items → print the usage with `eprintln!` and `exit(1)`
  (1 = "something went wrong", 0 = success). **Check this first**, before touching `args[1]`!
- `args[3]` would crash if it doesn't exist → `args.get(3)` returns an `Option` instead
- `scale`: `.map_or(8, ...)` → use the number if given, otherwise 8
- `size`: `.map(...)` → stays an `Option<u32>`, because "no size" means auto-crop (a behaviour, not a number)
- arguments are always text → `.parse()` turns `"4"` into `4`; the type annotation (`u32`) tells it which number type
- `src` = source (input), `dst` = destination (output)

**size vs scale** (⊙_⊙): size = how big the cut square is (in original pixels), scale = how many times bigger each pixel gets. Final size = size × scale (48 × 8 = 384). To turn a 64px icon into 1024px: `16 0` (no crop, ×16), NOT `16 1024`!

### 2. The `Frame` struct
```rust
struct Frame {
    img: RgbaImage, // the pixels
    ms: u32,        // how long the frame shows
}
```
Keeps each image together with its duration (Python used two separate lists that had to stay in sync).

### 3. Loading the frames (っ˘ω˘ς)
- **PNG:** a PNG is just one picture → `image::open(path)` loads it, `.to_rgba8()` turns it into an `RgbaImage`, return it as a list of one frame
  - `image::open` gives a `DynamicImage` ("some kind of image") → `.to_rgba8()` = "give it to me as RGBA"
  - `to_rgb8` (no **a**) would throw away the alpha → transparency gone, black background (╥﹏╥)
- **GIF:** open the file, wrap it in `BufReader`, decode every frame, collect them into a list
- `BufReader` reads the file in big chunks instead of byte by byte, then keeps it in memory → faster. Use it when reading a file in many small pieces. `into_frames()` only exists on buffered readers, so it won't even compile without it
- `GifDecoder` instead of `ImageReader::open().decode()`: `.decode()` only gives the first frame, but I need every frame + its delay
- `.map(|f| ...)` turns every library frame (`image::Frame`, which holds an image + a delay in the library's own format) into my own `Frame`. `.collect()` gathers them into a `Vec<Frame>`
- `f.delay().numer_denom_ms()` → the delay as a fraction (num = numerator, den = denominator), e.g. `(100, 1)` → `num / den` = 100 ms
- `den == 0` would crash (division by zero), so the default is 120 ms
- `f.into_buffer()` takes the pixels out and uses up `f`, so read the delay first

### 4. Bounding box (`bbox`) □
The smallest rectangle around every non-transparent pixel.

Example: a 6x5 image, `#` is a visible pixel.
```
      x→  0 1 2 3 4 5
y=0       . . . . . .
y=1       . . # . . .
y=2       . # # # . .
y=3       . . # . . .
y=4       . . . . . .
```
Leftmost x = 1, rightmost x = 3, top y = 1, bottom y = 3. That's the whole idea: the smallest x/y and the largest x/y among the visible pixels.

How the code finds it:
- `img.enumerate_pixels()` gives `(x, y, pixel)` for every pixel
- `pixel[3]` is alpha: 0 = transparent, 255 = opaque, anything in between = see-through. Anything above 0 counts as visible
- keep 4 "best so far" values: smallest x/y, biggest x/y
- smallest starts at `i64::MAX`, biggest at `i64::MIN`, so the first real pixel always replaces them (starting smallest at 0 would never update! (°ロ°))
- `let (x, y) = (x as i64, y as i64);` is **shadowing**: a new variable with the same name and a different type
- the right/bottom edge is **exclusive** (one past the last pixel) → `x1.max(x + 1)`, so width = `x1 - x0`. Same as Rust's `0..5` or Python's `range(0, 5)`
- `found` flag: no visible pixel at all → return `None`
- Tested with: the "plus" shape above → `Some((1, 1, 4, 4))`, empty image → `None`

### 5. Crop box (`crop_box`)
One square box that fits the drawing in **every** frame, so the character doesn't jump around or get cut off during the animation.

`size` has 3 cases:
- `None` → auto
- `Some(0)` → don't crop (`return None`)
- `Some(s)` → the user's size, e.g. `Some(32)` → 32x32

Steps:
1. `bbox` every frame; `filter_map` drops the empty ones (`None`)
2. all frames empty → `None`, nothing to crop around
3. combine all boxes: smallest left/top, biggest right/bottom (`.map(|b| b.0).min()`: take the first number of each box, find the smallest). `.unwrap()` is safe because I checked `is_empty()` first
4. auto size = the bigger of width/height + 2px margin on both sides, rounded up to a multiple of 8
5. center it horizontally: `center_x = (x0 + x1) / 2` (the middle of the drawing), `left = center_x - size / 2` (where the square starts)
6. bottom = 2px below the feet (`y1 + MARGIN`), top = `bottom - size`
7. return `Some((left, top, size))`: where the square is (`left`, `top`) and how big (`size`)

**Why `size` and `MARGIN` are `i64`:** not because they can be negative, but because they do math with positions that can be (`left` goes negative when the drawing is near the edge). Rust doesn't let you mix number types.

**Rounding up trick** (◎_◎;): `(need + 7) / 8 * 8`. Integer division rounds down, and adding 7 first pushes it to the next multiple. Nobody invents this, you look it up! On unsigned types, Rust has `n.next_multiple_of(8)` built in. Lesson: check the type's docs page before inventing tricks.

### 6. Crop, enlarge, save a PNG 
- `crop_window` makes an empty, fully transparent square (`RgbaImage::new(size, size)`) and pastes the original into it with `imageops::overlay(&mut output, img, -left, -top)`
- why `-left, -top`: overlay puts the original's top-left corner at that spot. To make original pixel `left` land at window pixel 0, slide the original `left` pixels to the left. Like sliding a drawing under a paper window
- **padding**: if the crop box sticks out past the canvas edge, part of the window has nothing to copy, so it stays transparent
- `crop_window` only cuts. `scale_up` enlarges, with `imageops::resize(..., FilterType::Nearest)` (nearest neighbor = copy the nearest pixel, never blend → crisp pixel art)
- `if let Some((left, top, sz)) = crop_box(...)`: crop only if there's a box. `None` skips cropping
- **handle the "maybe" once**: `crop_box` turns `Option<u32>` into a decision, `if let` opens it, and `crop_window` gets plain numbers

> tiny problem: running with the same output name silently overwrites the file. Idea for later: refuse if output == input, and ask `[y/N]` if the file exists (◔_◔)

### 7. Merge holds (`merge_holds`)
Clean up the frames before saving:
1. same picture as the previous frame? → don't keep a copy, add its time to the previous frame
2. empty frame? → drop it, also give its time to the previous frame
3. new picture? → keep it

```
before:  sit  sit  jump jump jump (empty)    6 × 100 ms
after:   sit       jump                      200 ms + 300 ms
```
Same total time, smaller file.
- takes `Vec<Frame>` (no `&`) → takes ownership, moves each frame into the new list (no copying)
- `result[last].ms += frame.ms` adds the time to the last kept frame
- `frame.img == result[last].img` compares size + every pixel

### 8. Build the palette (the colour key) 🎨
- `collect_colors`: every visible colour, once (`!colors.contains(&rgb)`)
- `build_lut` ("look-up table"): colour → number, starting at **1**, because **0 = transparent**
- GIF has no half-transparency, so a pixel is visible if `alpha >= VISIBLE_ALPHA` (128, the halfway point). Use the **same** const everywhere, or a colour can be missing from the table → crash
- more than 64 colours → probably already blurred → refuse

### 9. Write the GIF 🎞️
- `flat_palette`: the GIF wants the key as one flat row: `0,0,0, 255,0,0, 0,0,255` (slot 0 = dummy for transparent, then each colour's R, G, B)
- `to_indices`: every pixel → its number (`lut[&rgb]`), invisible → `0`
- `collect_colors` vs `to_indices`: "**which** colours are used?" (small list, once) vs "**what colour is each pixel?**" (one number per pixel, per frame). Like a paint-by-numbers key vs the numbered picture
- `gif::Encoder` writes the file; `gif::Frame` (the gif crate's frame, NOT my `Frame`) holds the numbers
- `delay = ms / 10` because GIF delays are in **1/100 s**, not ms!
- `transparent = Some(0)` → slot 0 is invisible
- `dispose = Background` → clear before the next frame, so no ghost trail
- `set_repeat(Infinite)` → loop forever

## Testing (๑•̀ㅂ•́)و✧
Tests are honestly fun.
- `cargo test` runs every test; `cargo test merge` runs only names containing "merge"; `-- --nocapture` shows `println!` output
- Rust puts **unit tests in the same file** (`#[cfg(test)] mod tests { use super::*; ... }` at the bottom). They can test private functions and are left out of the real program
- `assert_eq!(what_i_got, what_i_expect)`
- `img.clone()` when the same image goes into several `Frame`s
- `put_pixel` changes the image in place and returns nothing (`()`): methods with `&mut self` usually work like this
- PNG tests write a real file to `std::env::temp_dir()`, load it back, then delete it
- **if something is hard to test, it's doing too many things at once** → split it into small functions that take something in and give something back
- **a failing test can be wrong too!** Check which side is wrong before "fixing" the code

## Making it a real command 
- `cargo install --path .` builds pixel-up and copies it to `~/.cargo/bin`, so I can type `pixel-up` anywhere
- **re-run it after every code change**, or the old version keeps running (ಠ_ಠ)

## Finding things myself (•̀ᴗ•́)و
- `RgbaImage` is a **type alias** for `ImageBuffer<Rgba<u8>, Vec<u8>>`, so its methods are documented on `ImageBuffer`
- VS Code: type `RgbaImage::` to list every function, Cmd + click to jump to the definition, hover to read the docs
- on docs.rs: functions with `self` → call with `.`; without `self` → call with `::` (like `::new`)
- when stuck: full compiler message → `rustc --explain E0XXX` → docs.rs → the Book → search → ask
- the compiler's suggestions make it *compile*, not always do what I *meant*

Sources:
- [docs.rs: imageops (resize, overlay)](https://docs.rs/image/latest/image/imageops/index.html)
- [docs.rs: ImageBuffer (new, from_pixel, ...)](https://docs.rs/image/latest/image/struct.ImageBuffer.html)
- [docs.rs: gif crate](https://docs.rs/gif)
- [The Rust Book](https://doc.rust-lang.org/book/)

## Mistakes and how I fixed them (；´д｀)ゞ
- 2026-11-11: wrote `.to_rgb8()` instead of `.to_rgba8()`. One missing letter = no alpha = no transparency + type error
- 2026-11-11: forgot `.to_rgba8()` entirely → E0308, `image::open` gives a `DynamicImage`, not an `RgbaImage`
- 2026-11-10: tried pixel-up on a PNG → "Not a gif". It could write PNGs but not read them, so I added PNG input. Also used `16 1024` when I meant `16 0` (size is the crop, not the output!)
- 2026-11-09: renamed to `merged_frames` but kept using `frames` below → "borrow of moved value". Shadowing (`let frames = merge_holds(frames);`) makes this mistake impossible
- 2026-11-09: test expected blue `[0, 0, 255]` but I drew green `[0, 255, 0]`. The test was wrong, not the code!
- 2026-11-09: VS Code kept auto-importing `result` and `RgbImage` → unused import warnings. Watch the autocomplete popup
- 2026-11-08: `merge_holds` test failed (got 3 frames, expected 1): I forgot the "same picture" branch. The test caught it ✧
- 2026-11-08: assigned `put_pixel(...)` to a variable → it returns `()`, not an image. Also drew a transparent pixel `[0,0,0,0]`, which changes nothing, and forgot `#[test]`
- 2026-11-05: michi (my tester) came out with blurry stripes between colours. Not my code: my first-ever Clip Studio export was already scaled smoothly, and nearest neighbor faithfully enlarged the blur. Fix: export at 100%
- 2026-11-05: `RgbaImage([0,0,0,0])` → E0423. `Rgba` is ONE pixel, `RgbaImage` is a whole image
- 2026-11-05: read `args[1]` before checking `args.len()` → crashes instead of showing my message. Check first, then use
- 2026-11-05: put `struct`, `fn` and `const` inside `main`. It works, but tests can't see them. Definitions go at the top level; `let` stays inside functions
- 2026-11-04: `for f in frames` moves the Vec; use `&frames` if I need it after
- 2026-11-04: wrote `Option<i64, i64, i64, i64>` → E0107. `Option` holds ONE thing, so it needs a tuple: `Option<(i64, i64, i64, i64)>`
- 2026-11-04: wrote `x1.max(x) + 1` instead of `x1.max(x + 1)`. No compiler error, but the +1 piled up for every visible pixel (got 7 instead of 4). The compiler can't catch wrong math: test with an example where I already know the answer
- 2026-11-03: used `print!` instead of `println!` → no newlines

## Python vs Rust
- Python's `getbbox()` → I had to write `bbox()` myself
- `-(-n // 8) * 8` → `(n + 7) / 8 * 8` (or `n.next_multiple_of(8)`)
- `ImageChops.difference(a, b).getbbox() is None` → `a == b`
- two lists (frames + durations) → one `Frame` struct
- `None` can sneak in anywhere → `Option` forces me to handle it
- `sys.argv[0]` is the script name, `args[0]` is the program name. Same idea!



```
                  　　  ⋆｡˚ ꒰ঌ Thank you for your attention ໒꒱ ˚｡⋆
                      ✦　　　　　　　 ˚　　　        ₊　　　       ☆彡
                            ☾　　　　          ⋆　　　　     ✧　　　　      ˚
                    ₊　　　　　　　　　　　　　　　　                      ˚
                  　　　　　　　　    ∩＿∩　   /) /)
                        ⋆　　　　  （˶ᵔᴥᵔ˶）（˶•×•˶）ﾉ　　        ✧
                  　　　　　        （つ　と）（　 つ）
                  　     ＿＿,,..-‐''"￣￣￣￣￣￣￣""''‐-..,,＿＿
```