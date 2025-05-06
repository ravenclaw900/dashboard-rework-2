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
  "$asset_path/css/xterm-5.5.0.css"
)

./scripts/clean-css.bash "${css_assets[@]:1}" > "${css_assets[0]}"

cat "${js_assets[@]}" | gzip -9 > "$dist_path/main.js"
cat "${css_assets[@]}" | gzip -9 > "$dist_path/main.css"
cat "$asset_path/icons.svg" | gzip -9 > "$dist_path/icons.svg"

rm "${css_assets[0]}"
