#!/bin/bash
# アイコン生成スクリプト
# 事前に元となるPNG画像(1024x1024)をicon.pngとして用意してください

# macOS
sips -z 32 32 icon.png --out icons/32x32.png
sips -z 128 128 icon.png --out icons/128x128.png
sips -z 256 256 icon.png --out icons/128x128@2x.png

# iconutil でicnsを作成（macOSのみ）
# mkdir icon.iconset
# sips -z 16 16 icon.png --out icon.iconset/icon_16x16.png
# ... (他のサイズも同様)
# iconutil -c icns icon.iconset

echo "アイコンを生成しました"
echo "注意: icon.icns と icon.ico は手動で作成するか、"
echo "オンラインツール（https://cloudconvert.com/）を使用してください"
