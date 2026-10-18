# Vox roadmap

This roadmap is a proposal, not an implementation or release claim.

1. Keep the existing Shape speech workflow usable while Deck's independent application and
   interoperability work is completed.
2. Define a script-led project containing script text, speaker choices, cue material, synthesis
   settings and reviewable outputs. Decide the complete migration boundary before moving code.
3. Reuse and test Shape's script grammar, role readiness checks, repeats, pauses, cue assembly
   and audio validation. Preserve source attribution and historical projects.
4. Add independent authorization and truthful execution receipts. A new application must not
   impersonate Shape's Runtime identity or write a Shape database directly.
5. Provide inspect, validate, render, status and export operations for both humans and AI,
   including interruption recovery and exact target/version checks.
6. Connect specialist audio editing to Echo through supported material handoffs.

Persistent segment-level re-recording, pronunciation controls and further editing features need
their own product and validation decisions. Shape's current in-process retry cache is not a
persistent production timeline.

No migration, separate speech runtime or desktop application is authorized by this planning
scaffold alone.
