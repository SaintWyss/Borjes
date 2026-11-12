# Local Bevy image patch

This is the source of `bevy_image` 0.19.0, distributed under MIT OR Apache-2.0; both license texts are included. The only functional change is its optional `ruzstd` dependency constraint, raised from 0.8 to 0.9.1 to address RUSTSEC-2026-0344. The KTX2 decompression path must be validated in CI. Remove this patch when an upstream compatible Bevy release includes the fix.
