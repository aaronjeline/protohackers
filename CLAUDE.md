# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is an OCaml implementation of Protohackers challenges - network programming puzzles that involve building servers for various protocols. Each challenge is implemented as a separate executable that shares common networking utilities.

## Build System & Commands

**Build System:** Dune 3.16+ with opam package management

**Common Commands:**
- `dune build` - Build all executables
- `dune exec <challenge>` - Run a specific challenge (e.g. `dune exec prime`, `dune exec echoserv`)
- `dune test` - Run tests
- `opam install . --deps-only` - Install dependencies

**Key Dependencies:** lwt, lwt_unix, yojson

## Architecture

**Shared Infrastructure:**
- `lib/common.ml` - Core networking utilities with Lwt monadic helpers
  - Provides `main` function that sets up server on `0.0.0.0:1337`
  - Exports Lwt bind operators (`let*`) and common modules
  - Uses `IO.establish_server_with_client_socket` pattern

**Challenge Structure:**
- Each challenge is a separate executable in `bin/` 
- All servers bind to port 1337 on all interfaces
- Pattern: `open Protohackers.Common` → implement `server` function → call `main "name" server`

**Async I/O:**
- All networking uses Lwt with `let*` syntax for monadic operations
- Socket operations use `Lwt_unix` module
- Error handling with exceptions and `Lwt.catch`

**Protocol Handling:**
- JSON-based protocols use `yojson` for parsing/encoding
- Input reading uses fixed-size buffers with overflow protection
- Response sending handles partial writes with retry loops