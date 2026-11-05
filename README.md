# What I learned building pixel-up
A small command-line tool, written in Rust, that crops pixel GIF file and enlarges them with crisp without blurring pixels.

I built it for my pomodoro app's sprite animations: since Clip Studio Paint has limited scaling to your desired size while keeping the quality intact, so I export the gif at 100%, nearest-neighbor scaling mode and let the pixel-up do the enlarging, with also nearest-neighbor scaling.

> 🚧 Work in progress. PNG output works; animated GIF output is next.

## What it does

1. Loads every frame of a GIF
2. Finds the drawing and crops one square around it, the same for every frame
   (so the character doesn't jump out of frame)
3. Enlarges every pixel into a solid block (nearest neighbor, no new colours)
4. Saves the result (so far in the repo itself)


## Usage

```sh
cargo run --release -- <input.gif> <output.png|gif> [scale] [size]
```

| Argument | Meaning | Default |
| --- | --- | --- |
| `scale` | how many times bigger each pixel gets | 8 |
| `size` | size of the square crop, in original pixels; `0` = no crop | auto |

Examples:

```sh
cargo run --release -- michi.gif michi_big.png            # auto crop, 8x
cargo run --release -- michi.gif michi_32.png 4 32       # 32x32 crop, 4x
cargo run --release -- michi.gif michi_no_crop.png 2 0        # no crop, 2x
```
**Tip:** Make sure the input file is exported at 100%, crisp or nearest-neighbor mode. Pixel-up cannot undo blurs and low quality.
## What I learned

This is my first Rust project. See [`LEARNING.md`](LEARNING.md) for my notes
on the language and the mistakes I made along the way.
