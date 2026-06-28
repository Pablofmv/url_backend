Production Issues and Solutions
__

Incident #1

Error: text HTTP/2 500 relation "click_events" does not exist

Root Cause: The production database did not contain the required tables.

Local database:

links (YES)
click_events (YES)

Production database:

links (NO)
click_events (NO)

Solution: Implemented automatic SQLx migrations executed during startup.
__

Incident #2

Error: Leptos hydration panic

Root Cause: LocalResource caused hydration issues between server rendering and client rendering.

Solution

Replaced LocalResource with: signal, Effect::new, spawn_local

---

Lessons Learned

Production environments differ from local environments.
Database migrations should be automated.
Real-world debugging skills are essential for backend engineering.
