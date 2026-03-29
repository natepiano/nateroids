#!/bin/bash

# Zoom Dance - Automated camera sequence for Nateroids
# Reads configuration from zoom_dance.toml
# Usage: zoom_dance.sh [PORT]  (default: 15702)

PORT_OVERRIDE="${1:-15702}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONFIG_FILE="$SCRIPT_DIR/zoom_dance.toml"

if [ ! -f "$CONFIG_FILE" ]; then
  echo "Error: Config file not found at $CONFIG_FILE"
  exit 1
fi

# Parse TOML config using grep/awk
get_config() {
  local key=$1
  grep "^$key\s*=" "$CONFIG_FILE" | head -1 | awk -F'=' '{print $2}' | tr -d ' "' | tr -d "'"
}

# Load configuration (will be reloaded each iteration)
load_config() {
  SPACESHIP_CHANCE=$(get_config "spaceship_chance")
  PAUSED_DURATION=$(get_config "paused_duration")
  SPIN_WATCH_DURATION=$(get_config "spin_watch_duration")
  ZOOM_OUT_ANIMATION_MS=$(get_config "zoom_out_animation_ms")
  DISTANCE_WATCH_DURATION=$(get_config "distance_watch_duration")
  NO_DONUT_RETRY_DELAY=$(get_config "no_donut_retry_delay")
  MIN_DISTANCE=$(get_config "min_distance")
  MAX_DISTANCE=$(get_config "max_distance")
  PITCH_MIN=$(get_config "pitch_min")
  PITCH_MAX=$(get_config "pitch_max")
  FOCUS_OFFSET_CHANCE=$(get_config "focus_offset_chance")
  FOCUS_OFFSET_RANGE=$(get_config "focus_offset_range")
  PORT="$PORT_OVERRIDE"
  EASING=$(get_config "easing")
  DISTANCE_RANGE=$((MAX_DISTANCE - MIN_DISTANCE))
}

# Initial load
load_config

echo "Zoom Dance - Dynamic Configuration Mode"
echo "Edit zoom_dance.toml and changes will apply on next iteration"
echo ""

# Query for camera entity (done once at startup)
echo "Querying for camera entity..."
CAMERA=$(curl -s -X POST http://127.0.0.1:$PORT/jsonrpc -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "id": 1, "method": "world.query", "params": {"filter": {"with": ["bevy_lagrange::OrbitCam"]}, "data": {}}}' \
  | jq -r '.result[0].entity')

if [ -z "$CAMERA" ]; then
  echo "Error: Could not find camera entity with OrbitCam component"
  exit 1
fi

echo "Found camera entity: $CAMERA"
echo ""
echo "Initial Configuration:"
echo "  Port: $PORT"
echo "  Paused duration: ${PAUSED_DURATION}s"
echo "  Spin watch duration: ${SPIN_WATCH_DURATION}s"
echo "  Distance watch duration: ${DISTANCE_WATCH_DURATION}s"
echo "  Zoom out distance: ${MIN_DISTANCE}-${MAX_DISTANCE} units"
echo "  Spaceship chance: ${SPACESHIP_CHANCE}%"
echo ""

# Query physics pause state and sync
is_paused=$(curl -s -X POST http://127.0.0.1:$PORT/jsonrpc -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "id": 1, "method": "world.get_resources", "params": {"resource": "bevy_time::time::Time<avian3d::schedule::time::Physics>"}}' \
  | jq -r '.result.value.context.paused')

# If currently unpaused, send Esc to pause
if [ "$is_paused" = "false" ]; then
  echo "Game is running, pausing..."
  curl -s -X POST http://127.0.0.1:$PORT/jsonrpc -H "Content-Type: application/json" \
    -d '{"jsonrpc": "2.0", "id": 1, "method": "brp_extras/send_keys", "params": {"keys": ["Escape"], "duration_ms": 100}}' >/dev/null 2>&1
else
  echo "Game is already paused"
fi

echo "Starting zoom dance loop..."
echo ""

# Main loop
iteration=0
while true; do
  iteration=$((iteration + 1))

  # Reload configuration dynamically
  load_config

  # Clear target from previous iteration
  target=""

  echo "--- Iteration $iteration ---"

  # Decide target (spaceship or random donut)
  if (( RANDOM % 100 < SPACESHIP_CHANCE )); then
    # Query for spaceship (it can die and respawn with new entity ID)
    target=$(curl -s -X POST http://127.0.0.1:$PORT/jsonrpc -H "Content-Type: application/json" \
      -d '{"jsonrpc": "2.0", "id": 1, "method": "world.query", "params": {"filter": {"with": ["bevy_ecs::name::Name"]}, "data": {"components": ["bevy_ecs::name::Name"]}}}' \
      | jq -r '.result[] | select(.components."bevy_ecs::name::Name" == "Spaceship") | .entity' | head -1)

    if [ -z "$target" ]; then
      echo "Spaceship not found, falling back to donut"
      # Fall through to donut selection below
    else
      echo "Target: Spaceship ($target)"
    fi
  fi

  # If no spaceship target set, use a random donut
  if [ -z "$target" ]; then
    # Query for live donuts
    donuts=$(curl -s -X POST http://127.0.0.1:$PORT/jsonrpc -H "Content-Type: application/json" \
      -d '{"jsonrpc": "2.0", "id": 1, "method": "world.query", "params": {"filter": {"with": ["bevy_ecs::name::Name"]}, "data": {"components": ["bevy_ecs::name::Name"]}}}' \
      | jq -r '.result[] | select(.components."bevy_ecs::name::Name" == "donut") | .entity')

    if [ -z "$donuts" ]; then
      echo "No donuts found, waiting ${NO_DONUT_RETRY_DELAY}s..."
      sleep "$NO_DONUT_RETRY_DELAY"
      continue
    fi

    target=$(echo "$donuts" | awk -v r=$RANDOM 'BEGIN{srand(r)} {a[NR]=$0} END{print a[int(rand()*NR)+1]}')
    echo "Target: Donut $target"
  fi

  # Zoom in (while paused)
  curl -s -X POST http://127.0.0.1:$PORT/jsonrpc -H "Content-Type: application/json" \
    -d "{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"world.trigger_event\", \"params\": {\"event\": \"bevy_lagrange::ZoomToFit\", \"value\": {\"entity\": $CAMERA, \"target\": $target, \"margin\": 0.1, \"duration_ms\": 500.0}}}" >/dev/null 2>&1
  echo "Zoomed in (paused for ${PAUSED_DURATION}s)"

  sleep "$PAUSED_DURATION"

  # Unpause
  curl -s -X POST http://127.0.0.1:$PORT/jsonrpc -H "Content-Type: application/json" \
    -d '{"jsonrpc": "2.0", "id": 1, "method": "brp_extras/send_keys", "params": {"keys": ["Escape"], "duration_ms": 100}}' >/dev/null 2>&1
  echo "Unpaused (watching spin for ${SPIN_WATCH_DURATION}s)"

  sleep "$SPIN_WATCH_DURATION"

  # Generate random zoom-out position
  yaw=$(awk "BEGIN {srand(); print rand() * 6.28318}")
  pitch=$(awk "BEGIN {srand(); print ($PITCH_MIN) + rand() * (($PITCH_MAX) - ($PITCH_MIN))}")
  radius=$(echo "$MIN_DISTANCE + $RANDOM % $DISTANCE_RANGE" | bc)

  # Calculate translation
  x=$(echo "scale=2; $radius * s($yaw) * c($pitch)" | bc -l)
  y=$(echo "scale=2; $radius * s($pitch)" | bc -l)
  z=$(echo "scale=2; $radius * c($yaw) * c($pitch)" | bc -l)

  # Sometimes offset the focus to test off-center convergence
  if (( RANDOM % 100 < FOCUS_OFFSET_CHANCE )); then
    focus_x=$(awk -v range=$FOCUS_OFFSET_RANGE "BEGIN {srand(); print (rand() - 0.5) * range}")
    focus_y=$(awk -v range=$FOCUS_OFFSET_RANGE "BEGIN {srand(); print (rand() - 0.5) * range}")
    focus_z=$(awk -v range=$FOCUS_OFFSET_RANGE "BEGIN {srand(); print (rand() - 0.5) * range}")
    echo "Zooming out with offset focus: [$focus_x, $focus_y, $focus_z]"
  else
    focus_x=0
    focus_y=0
    focus_z=0
  fi

  # Zoom out
  curl -s -X POST http://127.0.0.1:$PORT/jsonrpc -H "Content-Type: application/json" \
    -d "{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"world.trigger_event\", \"params\": {\"event\": \"bevy_lagrange::PlayAnimation\", \"value\": {\"entity\": $CAMERA, \"moves\": [{\"target_translation\": [$x, $y, $z], \"target_focus\": [$focus_x, $focus_y, $focus_z], \"duration_ms\": $ZOOM_OUT_ANIMATION_MS, \"easing\": \"$EASING\"}]}}}" >/dev/null 2>&1
  echo "Zooming out (${ZOOM_OUT_ANIMATION_MS}ms animation, watching for ${DISTANCE_WATCH_DURATION}s)"

  sleep "$DISTANCE_WATCH_DURATION"

  # Pause for next iteration
  curl -s -X POST http://127.0.0.1:$PORT/jsonrpc -H "Content-Type: application/json" \
    -d '{"jsonrpc": "2.0", "id": 1, "method": "brp_extras/send_keys", "params": {"keys": ["Escape"], "duration_ms": 100}}' >/dev/null 2>&1
  echo "Paused (ready for next iteration)"
  echo ""
done
