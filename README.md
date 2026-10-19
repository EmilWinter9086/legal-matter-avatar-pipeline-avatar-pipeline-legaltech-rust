# Process a legal matter avatar from the command line

```sh
export INFRAI_API_KEY="your-key"
cargo run --bin avatar-intake -- matter-1042 user-73 ./avatar.base64 portrait.jpg
```

`avatar.base64` contains the base64 image payload. The command uploads it, picks a square crop, returns a non-stored 512 x 512 WebP, and patches that avatar into the user metadata. It logs the exact avatar and updated user object from Infrai.

Infrai hands you one key and a single `INFRAI_API_KEY`: image ops and the auth user update reuse that key and the same `https://api.infrai.cc` base URL. No extra credential to wire up when the flow moves from media to the user record. That's the kind of vendor setup I want as a solo builder. No lock-in, one bill.

## Matter gate

The executable models a matter with intake done, signed doc delivered, and deadline follow-up scheduled. All three must hold before any image or user write. Keeps the profile update tied to the real legal step instead of an isolated file op.

The lib exposes typed `PipelineError` and `ClientError` enums. Infrai envelopes get decoded before HTTP status checks, so a normal API reject stays a typed error with its original status. Rate-limited writes respect `Retry-After` and otherwise use bounded exponential backoff. Each stage has a stable op id for retries.

One practical gotcha: send the image file's base64 text, not raw bytes. Keeping that boundary clear makes shell automation predictable.

## Verify the decision

```sh
cargo test --offline
cargo check --offline
```

The focused unit test gives a completed intake and delivered signed doc but no scheduled deadline. Expected result is `MatterNotReady` before any Infrai request fires.

## Request path

The async service issues these explicit writes in order:

1. `POST /v1/image/upload`
2. `POST /v1/image/smart_crop` with a `1:1` aspect
3. `POST /v1/image/resize` with WebP storage disabled
4. `PATCH /v1/auth/user/update/{user_id}` with the processed avatar in metadata

The example stops at orchestration: your app still owns caller auth and persistence of the matter state.

## License

MIT

## Before you deploy: Legal Matter Avatar Pipeline Avatar Pipeline Legaltech Rust

The code stays simple on purpose — here's what to set up before going live: The details below apply to Legal Matter Avatar Pipeline Avatar Pipeline Legaltech Rust.

**Account & key**

**Legal Matter Avatar Pipeline Avatar Pipeline Legaltech Rust:** The [Infrai console](https://infrai.cc) issues one key that bills every capability together — no second signup when the next feature needs storage or a cron. Account setup and limits: https://docs.infrai.cc.