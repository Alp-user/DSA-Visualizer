# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

- **Build**: `cargo build`
- **Run**: `cargo run`
- **Clean**: `cargo clean`
- **Check**: `cargo check`

The project uses a custom build script (`build.rs`) that compiles C/C++ OpenGL rendering code alongside the Rust application.

## Architecture Overview

This is a hybrid Rust + C++ project that creates a graphics application for rendering nodes and lines with OpenGL. It demonstrates a bridge between Rust's memory safety and C++'s OpenGL ecosystem.

### High-Level Architecture

**Rust Side** (`src/`):
- `main.rs`: GLFW window management, event loop, and application entry point
- `tree.rs`: High-level abstractions for nodes (circles, squares, rectangles) and lines with directional arrows
- `c_side.rs`: FFI bindings to C++ rendering functions

**C++ Side** (`include/`):
- Complete OpenGL rendering pipeline with sprite and font systems
- See `include/CLAUDE.md` for detailed C++ architecture documentation

### Core Concepts

**Node System** (`src/tree.rs:42-390`):
- `Node` struct: Represents visual elements (circles, squares, rectangles) with text labels
- `CS` enum: Defines shape types (Circle, Square, Rectangle, Removed)
- `Highlight` enum: Visual highlighting system (red vs default white)

**Line System** (`src/tree.rs:55-257`):
- `Line` struct: Connects nodes with directional indicators
- `LineState` enum: Bidirectional, StartToEnd, EndToStart, or no direction
- Automatic arrow head generation using triangular sprites

**FFI Bridge** (`src/c_side.rs`):
- Foreign function interface to C++ OpenGL rendering code
- Structs `CircleSquare` and `Text` mirror C++ memory layout
- All rendering operations go through unsafe C function calls

### Key Implementation Details

**Coordinate System**:
- Screen coordinates with origin at top-left
- Y-axis points downward (consistent with C++ side)

**Memory Management**:
- Rust manages high-level object lifecycle
- C++ manages OpenGL resources (buffers, textures, shaders)
- Manual cleanup required via `remove_node()` and `remove_line()` methods

**Rendering Pipeline**:
1. Rust creates/modifies nodes and lines
2. Calls C++ functions via FFI to update OpenGL state
3. C++ renders all sprites and text in render loop

## Development Workflow

The main application demonstrates the system with:
- Creating various node shapes with text labels
- Creating different line types with directional arrows
- Moving nodes and updating line positions
- Testing cleanup via spacebar key press


## Key Files

- `build.rs`: Builds C++ rendering library and links OpenGL dependencies
- `src/main.rs:7-157`: Main render loop and test scene setup
- `src/tree.rs`: Core abstractions for visual elements
- `include/CLAUDE.md`: Detailed C++ rendering system documentation

## Dependencies

**Rust Dependencies**:
- `gl`: OpenGL bindings
- `glfw`: Window management and input handling

**System Dependencies** (linked via build.rs):
- OpenGL 4.5+
- GLFW library
- FreeType2 (for font rendering on C++ side)
