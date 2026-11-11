# pixel-up
A small command-line tool, written in Rust, that crops pixel GIF file and enlarges them with crisp without blurring pixels.

I built it for my pomodoro app's sprite animations: since Clip Studio Paint has limited scaling to your desired size while keeping the quality intact, so I export the gif at 100%, nearest-neighbor scaling mode and let the pixel-up do the enlarging, with also nearest-neighbor scaling.

## What it does

1. Loads every frame of a GIF, or a single .png image (new feature)
2. Finds the drawing and crops one square around it, the same for every frame
   (so the character doesn't jump out of frame)
3. Enlarges every pixel into a solid block (nearest neighbor, no new colours)
4. Saves the result as a PNG or an animated GIF

## Install

You need [Rust](https://www.rust-lang.org/tools/install). Then, inside this repo:

```sh
cargo install --path .
```

This builds pixel-up and copies it to `~/.cargo/bin`, so you can run `pixel-up`
from any folder. (Make sure `~/.cargo/bin` is in your `PATH`; the Rust
installer usually adds it.) After changing the code, run the same command
again to update it.

## Usage

```sh
pixel-up <input.gif|png> <output.gif|png> [scale] [size]
```

| Argument | Meaning | Default |
| --- | --- | --- |
| `scale` | how many times bigger each pixel gets | 8 |
| `size` | size of the square crop, in original pixels; `0` = no crop | auto |

Examples:

```sh
pixel-up michi.gif michi_big.png            # auto crop, 8x
pixel-up michi.gif michi_32.png 4 32       # 32x32 crop, 4x
pixel-up michi.gif michi_no_crop.png 2 0        # no crop, 2x
pixel-up ratito-icon64.png ratito-icon1024.png 16 0   # 64px icon → 1024px
```
Without installing, use `cargo run --release --` instead of `pixel-up`.

**Tip:** export your art at 100%, with crisp or nearest-neighbor scaling.
pixel-up can't undo blur, and it will refuse images with more than 64 colours,
which usually means they were already scaled smoothly.

## What I learned ｡˚✦⋆ ٩(◕ᴗ◕)۶ ⋆✦˚｡

This is my first Rust project. See [`LEARNING.md`](LEARNING.md) for my notes
on the language and the mistakes I made along the way.