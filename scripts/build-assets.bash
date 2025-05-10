#!/bin/bash -ex

asset_path='crates/server/assets'
dist_path='crates/server/dist'

js_assets=(
  "$asset_path/js/xterm-5.5.0.js"
  "$asset_path/js/microlight-0.0.7.js"
  "$asset_path/js/components.js"
)

css_assets=(
  "$asset_path/css/vars-clean.css"
  "$asset_path/css/global.css"
  "$asset_path/css/system.css"
  "$asset_path/css/process.css"
  "$asset_path/css/xterm-5.5.0.css"
)

js_out="$dist_path/main.js"
css_out="$dist_path/main.css"
svg_out="$dist_path/icons.svg"

if command -v md5 > /dev/null; then
  md5_cmd='md5 -q'
elif command -v md5sum > /dev/null; then
  md5_cmd='md5sum'
else
  echo 'Missing md5 command'
  exit 1
fi

./scripts/clean-css.bash "${css_assets[@]:1}" > "${css_assets[0]}"

cat "${js_assets[@]}" | gzip -9 > "$js_out"
cat "${css_assets[@]}" | gzip -9 > "$css_out"
cat "$asset_path/icons.svg" | gzip -9 > "$svg_out"

for file in "$js_out" "$css_out" "$svg_out"; do
  sum=$($md5_cmd "$file" | cut -d ' ' -f 1)
  echo -n "\"$sum\"" > "$file.md5"
done

rm "${css_assets[0]}"
