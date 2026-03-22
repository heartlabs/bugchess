# Capturing Piece Animation GIFs

This directory contains animated GIFs showcasing each piece type's creation and abilities.

## Files

- `piece_basic.gif` — Basic piece placement and movement
- `piece_cross.gif` — Cross formation merging and shield ability
- `piece_bar.gif` — Bar formation and gunfire ability
- `piece_queen.gif` — Queen formation and burst ability
- `piece_sniper.gif` — Sniper formation and long-range shot
- `piece_castle.gif` — Castle formation and protection ability

## How to Create These GIFs

### Step 1: Start the Capture Script

In the project root, run:

```bash
npm --prefix automation/playwright install  # if not done yet
BaseURL=http://localhost:4000/index.htm npm --prefix automation/playwright run capture-pieces
```

This opens a visible browser window with Bugchess in offline mode, ready for recording.

### Step 2: Record Gameplay with Your Screen Recorder

Use one of these tools:

**macOS:**

```bash
# QuickTime Player (built-in)
# File → New Screen Recording, then record your gameplay
```

**macOS (command line with ffmpeg):**

```bash
ffmpeg -f avfoundation -i "1" -t 15 piece_cross_raw.mov
```

**Linux/Windows with OBS:**

- Open OBS Studio
- Add "Screen Capture" source
- Record 15-20 second clips for each piece

### Step 3: Demonstrate Each Piece

For each piece, perform these actions:

**Basic Piece:**

- Click an empty square to place a basic piece
- Click and drag to move it
- Show movement range when hovering

**Cross (+):**

- Place 5 basic pieces in a cross pattern
- Show the merge animation
- Click twice on the cross to (optional) demonstrate shield

**Bar (— or |):**

- Place 3 basic pieces in a horizontal or vertical line
- Show the merge animation
- Click twice to activate gunfire ability
- Show bullets destroying pieces

**Queen (◊):**

- Place 8 basic pieces in a diamond/rotated square pattern
- Show the merge animation
- Click twice to activate burst
- Show neighboring pieces being destroyed

**Sniper (X):**

- Place 5 basic pieces in an X pattern
- Show the merge animation
- Click twice to activate long-range shot
- Target a piece anywhere on the board

**Castle (◽):**

- Place 4 basic pieces in a small rotated square
- Show the merge animation
- Place enemy pieces nearby
- Demonstrate protection (if ability is passive)

### Step 4: Convert Video to GIF

Once you have your recordings, convert them to optimized GIFs:

```bash
# Basic conversion (macOS)
ffmpeg -i piece_cross_raw.mov \
  -vf "fps=10,scale=400:-1:flags=lanczos" \
  -c:v pam -f image2pipe - | \
  convert -delay 10 - -loop 0 -colors 256 piece_cross.gif

# Or simpler (may be larger file size)
ffmpeg -i piece_cross_raw.mov \
  -vf "fps=10,scale=400:-1" \
  piece_cross.gif

# For multiple files, loop through:
for piece in cross bar queen sniper castle; do
  ffmpeg -i piece_${piece}_raw.mov \
    -vf "fps=10,scale=400:-1" \
    piece_${piece}.gif
done
```

### Step 5: Place GIFs in html/gifs/

```bash
mkdir -p html/gifs
mv piece_*.gif html/gifs/
```

## Tips for Better GIFs

- **Framerate:** 8-12 fps often looks smooth enough while keeping file size down
- **Scale:** 300-400px wide fits nicely in the page layout
- **Duration:** Keep each GIF to 5-15 seconds max
- **Quality:** Use `scale=400:-1` to maintain aspect ratio
- **File Size:** Optimize with `colors=256` to reduce file size

## Alternative: Use ffmpeg with Video Pre-processing

For lower file sizes with better quality:

```bash
# Convert video → intermediate -> GIF (using ImageMagick)
ffmpeg -i recording.mov -vf "fps=10,scale=400:-1" frame_%03d.png
convert -delay 10 frame_*.png -loop 0 -colors 256 piece_cross.gif
rm frame_*.png
```
