Read the brief at <scratchpad>/porting-brief.md first and follow it.

Your Ink files (check exact extensions in the test dir): width-height, padding, margin, gap, flex, flex-direction, flex-wrap, flex-align-items, flex-align-self, flex-align-content, flex-justify-content. Write one tests/<name>.rs per file (dashes → underscores).

Upstream skips you will meet: two `space-around` text-node tests in flex-justify-content ("Yoga bug"), and "min width in percent" / "max width in percent" in width-height ("Yoga does not support"). Port them with `#[ignore = "skipped upstream: …"]`. Their expectations were not captured (skipped tests do not run), so take the expected value from the Ink source.
