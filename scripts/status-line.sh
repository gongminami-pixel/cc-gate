#!/bin/bash
# CC-Gate status line script — called by Claude Code via statusLine.command
# Reads JSON from stdin, outputs formatted status line to stdout
# Model pricing and context window sizes are hardcoded because Claude Code
# doesn't correctly read context_window from gateway /v1/models responses.

fmt_tokens(){ local n=$1; if [ "$n" -ge 1000000 ]; then local m=$(awk -v n="$n" 'BEGIN { printf "%.1f", n/1000000 }'); echo "${m}M"; elif [ "$n" -ge 10000 ]; then echo "$((n/1000))k"; elif [ "$n" -ge 1000 ]; then local k=$(awk -v n="$n" 'BEGIN { printf "%.1f", n/1000 }'); echo "${k}k"; else echo "$n"; fi; }
input=$(cat)
cwd=$(echo "$input" | jq -r '.cwd // empty')
[ -n "$cwd" ] && dir_display="${cwd/#$HOME/~}" || dir_display="?"
model_id=$(echo "$input" | jq -r '.model.id // empty')
model_display=$(echo "$input" | jq -r '.model.display_name // empty')
total_input=$(echo "$input" | jq -r '.context_window.total_input_tokens // empty')
total_output=$(echo "$input" | jq -r '.context_window.total_output_tokens // empty')
window_size=$(echo "$input" | jq -r '.context_window.context_window_size // empty')
used_pct=$(echo "$input" | jq -r '.context_window.used_percentage // empty')
input_price=0; output_price=0
case "$model_id" in
  *opus*) input_price=15; output_price=75;;
  *deepseek*) input_price=1.50; output_price=6;;
  *glm*) input_price=0.50; output_price=2;;
  *qwen*) input_price=0.50; output_price=2;;
  *mimo*) input_price=0.50; output_price=2;;
  *gpt-5.1*|*gpt5.1*) input_price=3; output_price=15;;
  *sonnet*) input_price=3; output_price=15;;
  *haiku*) input_price=0.80; output_price=4;;
esac
case "$model_id" in
  *deepseek*) window_size=1000000;;
  *glm*) window_size=1000000;;
  *qwen*) window_size=1048576;;
  *mimo*) window_size=1000000;;
esac
if [ -n "$total_input" ] && [ -n "$window_size" ]; then
  used_pct=$(awk -v ti="$total_input" -v ws="$window_size" 'BEGIN { printf "%.0f", (ti/ws)*100 }')
fi
model_label="${model_display:-$model_id}"
out=''
[ -n "$model_label" ] && out="$(printf '\033[1;36m%s\033[0m' "$model_label")"
if [ -n "$out" ] && [ -n "$dir_display" ]; then out="${out} | ${dir_display}"
elif [ -z "$out" ] && [ -n "$dir_display" ]; then out="$dir_display"; fi
if [ -n "$total_input" ]; then
  ctx="ctx: $(fmt_tokens $total_input)"
  if [ -n "$window_size" ]; then ctx="${ctx}/$(fmt_tokens $window_size)"; fi
  if [ -n "$used_pct" ]; then ctx="${ctx} $(printf '%.0f' "$used_pct")%"; fi
  [ -n "$out" ] && out="${out} | ${ctx}" || out="$ctx"
fi
ti=${total_input:-0}; to=${total_output:-0}
if [ "$input_price" != "0" ] || [ "$output_price" != "0" ]; then
  cost=$(echo "scale=4; ($ti/1000000)*$input_price + ($to/1000000)*$output_price" | bc 2>/dev/null)
  [ -z "$cost" ] && cost=$(awk -v ti="$ti" -v to="$to" -v ip="$input_price" -v op="$output_price" 'BEGIN { printf "%.4f", (ti/1000000)*ip + (to/1000000)*op }')
  cost_fmt=$(printf '$%.2f' "$cost" 2>/dev/null)
  [ -z "$cost_fmt" ] && cost_fmt='$0.00'
  [ -n "$out" ] && out="${out} | ${cost_fmt}" || out="$cost_fmt"
fi
printf '%s' "$out"
