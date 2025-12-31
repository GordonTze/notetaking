#!/bin/bash

echo "Building Notetaking App..."
echo "=========================="
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null
then
    echo "❌ Rust is not installed!"
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

echo "✓ Rust found: $(rustc --version)"
echo ""

# Build in release mode
echo "Building application (this may take a few minutes)..."
cargo build --release

if [ $? -eq 0 ]; then
    echo ""
    echo "✓ Build successful!"
    echo ""
    echo "To run the application:"
    echo "  ./target/release/notetaking-app"
    echo ""
    echo "Or use:"
    echo "  cargo run --release"
    echo ""
    
    # Ask if user wants to run now
    read -p "Run the application now? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]
    then
        ./target/release/notetaking-app
    fi
else
    echo ""
    echo "❌ Build failed!"
    exit 1
fi
