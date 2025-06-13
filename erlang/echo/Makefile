# Makefile for Erlang echo server

# Find all .erl files
ERL_SOURCES = $(wildcard *.erl)
BEAM_FILES = $(ERL_SOURCES:.erl=.beam)

# Default target
all: $(BEAM_FILES)

# Rule to compile .erl files to .beam files
%.beam: %.erl
	erlc $<

# Clean target to remove compiled files
clean:
	rm -f *.beam
	rm -f erl_crash.dump

.PHONY: all clean