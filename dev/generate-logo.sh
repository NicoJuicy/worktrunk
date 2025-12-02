#!/usr/bin/env bash
# Generate the Worktrunk logo from the JSON prompt
#
# Requirements:
#   - gemimg: uv tool install gemimg (or run with: uvx gemimg)
#   - imagemagick: brew install imagemagick
#
# Usage:
#   ./dev/generate-logo.sh
#
# This generates multiple variants and you pick the best one.
# The script applies rounded corners and creates both 1x (512px) and 2x (1024px) versions.

set -euo pipefail

cd "$(dirname "$0")/.."

PROMPT_FILE="dev/logo-prompt.json"
OUTPUT_DIR="."
SIZE_1X=512
SIZE_2X=1024
CORNER_RADIUS_1X=48
CORNER_RADIUS_2X=96

if [[ ! -f "$PROMPT_FILE" ]]; then
    echo "Error: $PROMPT_FILE not found"
    exit 1
fi

if ! command -v gemimg &> /dev/null; then
    echo "Error: gemimg not found. Install with: uv tool install gemimg"
    exit 1
fi

if ! command -v magick &> /dev/null; then
    echo "Error: imagemagick not found. Install with: brew install imagemagick"
    exit 1
fi

echo "Generating 3 logo variants..."
for i in 1 2 3; do
    echo "  Generating variant $i..."
    gemimg "$(cat "$PROMPT_FILE")" \
        --model gemini-3-pro-image-preview \
        --aspect-ratio 1:1 \
        -o "$OUTPUT_DIR/logo-variant-$i.png"
done

echo ""
echo "Applying rounded corners (1x and 2x versions)..."
for i in 1 2 3; do
    # 1x version (512px)
    magick "$OUTPUT_DIR/logo-variant-$i.png" -resize "${SIZE_1X}x${SIZE_1X}" \
        \( +clone -alpha extract \
            -draw "fill black polygon 0,0 0,$CORNER_RADIUS_1X $CORNER_RADIUS_1X,0 fill white circle $CORNER_RADIUS_1X,$CORNER_RADIUS_1X $CORNER_RADIUS_1X,0" \
            \( +clone -flip \) -compose Multiply -composite \
            \( +clone -flop \) -compose Multiply -composite \
        \) -alpha off -compose CopyOpacity -composite \
        "$OUTPUT_DIR/logo-variant-$i-rounded.png"

    # 2x version (1024px)
    magick "$OUTPUT_DIR/logo-variant-$i.png" -resize "${SIZE_2X}x${SIZE_2X}" \
        \( +clone -alpha extract \
            -draw "fill black polygon 0,0 0,$CORNER_RADIUS_2X $CORNER_RADIUS_2X,0 fill white circle $CORNER_RADIUS_2X,$CORNER_RADIUS_2X $CORNER_RADIUS_2X,0" \
            \( +clone -flip \) -compose Multiply -composite \
            \( +clone -flop \) -compose Multiply -composite \
        \) -alpha off -compose CopyOpacity -composite \
        "$OUTPUT_DIR/logo-variant-$i-rounded@2x.png"
done

echo ""
echo "Generated files:"
ls -la "$OUTPUT_DIR"/logo-variant-*.png

echo ""
echo "Review the *-rounded.png variants and copy the best one:"
echo "  cp logo-variant-N-rounded.png docs/static/logo.png"
echo "  cp logo-variant-N-rounded@2x.png docs/static/logo@2x.png"
echo ""
echo "Then clean up:"
echo "  rm logo-variant-*.png"
