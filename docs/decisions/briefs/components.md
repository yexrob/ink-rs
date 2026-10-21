Read the brief at <scratchpad>/porting-brief.md first and follow it.

Your Ink file: test/components.tsx → tests/components.rs.

It is the largest file and mixes static rendering with runtime tests (rerender, hooks, stdin, etc.). Port the static ones; list every runtime one in the report with its milestone.
