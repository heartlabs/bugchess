#!/bin/bash
# Create animated GIFs for Bugchess landing page directly from the in-game pieces sprite sheet.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SPRITE_SHEET="$ROOT_DIR/game-render/resources/sprites/insekten4.png"
OUTPUT_DIR="$ROOT_DIR/html/gifs"

mkdir -p "$OUTPUT_DIR"

if ! command -v ffmpeg >/dev/null 2>&1; then
    echo "❌ ffmpeg is required but not installed."
    exit 1
fi

if ! command -v ffprobe >/dev/null 2>&1; then
    echo "❌ ffprobe is required but not installed."
    exit 1
fi

if [[ ! -f "$SPRITE_SHEET" ]]; then
    echo "❌ Sprite sheet not found: $SPRITE_SHEET"
    exit 1
fi

echo "🎬 Creating animated piece GIFs from sprite sheet..."

# Coordinates must stay in sync with SpriteRender::piece_sprite_rect in game-render/src/sprite.rs
# Each crop is 295x255 from offsets:
#   x = sprite_x * 295 + 250
#   y = sprite_y * 255 + 100
create_piece_gif() {
    local piece_name="$1"
    local crop_x="$2"
    local crop_y="$3"
    local output="$OUTPUT_DIR/piece_${piece_name}.gif"

    echo "Creating piece_${piece_name}.gif..."

    ffmpeg -v error -y \
        -loop 1 -t 2.2 -i "$SPRITE_SHEET" \
        -filter_complex "[0:v]crop=295:255:${crop_x}:${crop_y},scale=360:312:flags=lanczos,pad=420:360:(ow-iw)/2:(oh-ih)/2:color=0x1a1a1a,rotate='0.02*sin(2*PI*t/2.2)':c=0x1a1a1a,fps=12,split[s0][s1];[s0]palettegen=stats_mode=single[p];[s1][p]paletteuse=dither=sierra2_4a" \
        "$output"

    local frames
    frames="$(ffprobe -v error -count_frames -select_streams v:0 -show_entries stream=nb_read_frames -of csv=p=0 "$output" | tr -d '[:space:]')"
    if [[ -z "$frames" || "$frames" -le 1 ]]; then
        echo "❌ Validation failed for piece_${piece_name}.gif: expected >1 frame, got ${frames:-0}"
        exit 1
    fi

    local size
    size="$(du -h "$output" | awk '{print $1}')"
    echo "✓ piece_${piece_name}.gif (${size}, ${frames} frames)"
}

# Piece crop positions from SpriteRender::piece_sprite_rect mapping
create_piece_gif "basic" 250 100      # PieceKind::Simple (0,0)
create_piece_gif "cross" 840 100      # PieceKind::Cross (2,0)
create_piece_gif "bar" 840 355        # PieceKind::HorizontalBar / VerticalBar (2,1)
create_piece_gif "queen" 545 355      # PieceKind::Queen (1,1)
create_piece_gif "sniper" 250 355     # PieceKind::Sniper (0,1)
create_piece_gif "castle" 250 610     # PieceKind::Castle (0,2)

echo ""
echo "✅ GIF creation complete!"
echo ""
echo "Files created:"
ls -lh "$OUTPUT_DIR"/piece_*.gif 2>/dev/null | awk '{print "   " $NF " (" $5 ")"}'
