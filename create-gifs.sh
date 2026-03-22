#!/bin/bash
# Create animated GIFs from captured frames for Bugchess pieces using ffmpeg

set -e

FRAMES_DIR="/Users/neidhartorlich/dev/personal/megachess/html/gifs/frames"
OUTPUT_DIR="/Users/neidhartorlich/dev/personal/megachess/html/gifs"
TEMP_DIR=$(mktemp -d)

cleanup() {
    rm -rf "$TEMP_DIR"
}
trap cleanup EXIT

echo "🎬 Creating animated GIFs from captured frames..."

# Helper function to create a GIF from a single image by repeating it
create_single_frame_gif() {
    local input=$1
    local output=$2
    local duration=${3:-3}
    
    echo "Creating $(basename $output)..."
    
    # Use ffmpeg to convert single image to short video then to GIF
    # This creates a static-looking GIF from a single frame
    ffmpeg -loop 1 -i "$input" \
           -c:v libx264 -t $duration -pix_fmt yuv420p \
           -vf "scale=400:min(300\,(400*ih/iw))" \
           "$TEMP_DIR/$(basename ${output%.gif}).mp4" -y 2>/dev/null
    
    ffmpeg -i "$TEMP_DIR/$(basename ${output%.gif}).mp4" \
           -vf "fps=10,scale=400:-1" \
           "$output" -y 2>/dev/null
    
    echo "✓ $(basename $output) ($(du -h $output | cut -f1))"
}

# Create GIFs from each captured frame
create_single_frame_gif "$FRAMES_DIR/basic_01_placement.png" "$OUTPUT_DIR/piece_basic.gif" 2
create_single_frame_gif "$FRAMES_DIR/cross_01_merged.png" "$OUTPUT_DIR/piece_cross.gif" 2
create_single_frame_gif "$FRAMES_DIR/bar_01_merged.png" "$OUTPUT_DIR/piece_bar.gif" 2
create_single_frame_gif "$FRAMES_DIR/queen_01_partial.png" "$OUTPUT_DIR/piece_queen.gif" 2

# For missing pieces, use ffmpeg to create simple colored placeholders
echo ""
echo "Creating placeholder GIFs for pieces without direct captures..."

for piece in sniper castle; do
    # Create a simple color palette for placeholder
    ffmpeg -f lavfi -i color=c=#1a1a1a:s=400x300:d=3 \
           -vf "drawtext=fontsize=24:fontcolor=#666666:font=Arial:text='Recording $piece piece...':x=(w-text_w)/2:y=(h-text_h)/2,fps=10,scale=400:-1" \
           "$OUTPUT_DIR/piece_${piece}.gif" -y 2>/dev/null
    
    echo "✓ $OUTPUT_DIR/piece_${piece}.gif (placeholder)"
done

echo ""
echo "✅ GIF creation complete!"
echo ""
echo "Files created:"
ls -lh "$OUTPUT_DIR"/piece_*.gif 2>/dev/null | awk '{print "   " $NF " (" $5 ")"}'
