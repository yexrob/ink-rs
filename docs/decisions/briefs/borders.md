Read the brief at <scratchpad>/porting-brief.md first and follow it.

Your Ink files: test/borders.tsx, test/border-backgrounds.tsx, test/background.tsx (check the exact extensions in the test dir). Write tests/borders.rs, tests/border_backgrounds.rs, tests/background.rs.

Note: Ink's borders test near line 823 uses a custom BoxStyle object; that maps to `BorderStyle::Custom(BoxChars { … })`.
