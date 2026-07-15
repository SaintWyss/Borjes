# Security policy

Borjes treats dependency advisories as release-blocking unless a narrow,
documented exception proves that the vulnerable code is not reachable in the
shipped application.

## Reporting a vulnerability

Please open a private GitHub security advisory for this repository. Include the
affected version, reproduction steps, expected impact, and any suggested
mitigation. Avoid publishing exploit details before a fix is available.

## Current dependency disposition

### Patched

- **RUSTSEC-2026-0204 (`crossbeam-epoch`)**: resolved by locking
  `crossbeam-epoch` to 0.9.20.

### Temporary build-time exception

- **RUSTSEC-2026-0194 and RUSTSEC-2026-0195 (`quick-xml`)**
- Affected lock entry: `quick-xml` 0.39.4.
- Introduced only by: `wayland-scanner` 0.31.10.
- Runtime exposure: none. `wayland-scanner` is a procedural macro/code
  generator and parses the trusted Wayland protocol XML shipped by upstream
  crates during compilation. Borjes does not accept or parse XML from users,
  files, sockets, or the network.
- Risk accepted: denial of service against a build that first allows an
  attacker to replace trusted dependency source/protocol inputs. Such an
  attacker already controls build inputs and is outside the application threat
  model.
- Compensating controls: `Cargo.lock` is committed, CI builds use `--locked`,
  and the exception names exactly two advisory IDs rather than disabling the
  audit.
- Upstream status: Smithay/wayland-rs merged the `quick-xml` 0.41 migration
  after `wayland-scanner` 0.31.10.
- Removal condition: update the Wayland dependency family as soon as crates.io
  publishes a compatible release using `quick-xml >= 0.41.0`, then delete the
  CI `ignore` entry.
- Review deadline: **2026-08-15**, or immediately when a compatible release is
  published, whichever comes first.

Upstream references:

- https://github.com/Smithay/wayland-rs/pull/938
- https://rustsec.org/advisories/RUSTSEC-2026-0194.html
- https://rustsec.org/advisories/RUSTSEC-2026-0195.html

Informational/unmaintained or yanked transitive crates remain visible in audit
output; they are not hidden by this exception and should be removed through
normal Bevy ecosystem upgrades.
