#!/bin/bash

# A test script to verify your audio playback system works

echo "Testing audio playback system..."

# Check for media players
echo "Checking for media players..."
which mpv >/dev/null && echo "✓ mpv found" || echo "✗ mpv not found"
which ffplay >/dev/null && echo "✓ ffplay found" || echo "✗ ffplay not found"
which aplay >/dev/null && echo "✓ aplay found" || echo "✗ aplay not found"

# Generate a test tone using sox (if available)
if which sox >/dev/null; then
    echo "Generating test tone with sox..."
    sox -n /tmp/test_tone.mp3 synth 2 sine 440
    
    # Try to play with each player
    echo "Playing with mpv..."
    mpv /tmp/test_tone.mp3 --no-terminal >/dev/null 2>&1 && echo "✓ mpv playback works" || echo "✗ mpv playback failed"
    
    if which ffplay >/dev/null; then
        echo "Playing with ffplay..."
        ffplay -nodisp -autoexit -loglevel quiet /tmp/test_tone.mp3 >/dev/null 2>&1 && echo "✓ ffplay playback works" || echo "✗ ffplay playback failed"
    fi
    
    if which aplay >/dev/null; then
        # Convert to wav for aplay
        sox /tmp/test_tone.mp3 /tmp/test_tone.wav
        echo "Playing with aplay..."
        aplay /tmp/test_tone.wav >/dev/null 2>&1 && echo "✓ aplay playback works" || echo "✗ aplay playback failed"
    fi
else
    echo "Sox not found, skipping audio playback tests"
    echo "Install sox with: sudo apt-get install sox (Debian/Ubuntu)"
fi

# Test API endpoint
echo "Testing TTS API endpoint..."
curl -s "http://0.0.0.0:8000/v1/audio/models" | grep model && echo "✓ API is responding" || echo "✗ API not responding"

echo "Done!"
