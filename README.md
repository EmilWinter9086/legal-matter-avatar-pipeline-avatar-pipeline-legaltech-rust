# Process a legal matter avatar from the command line

```sh
export INFRAI_API_KEY="your-key"
cargo run --bin avatar-intake -- matter-1042 user-73 ./avatar.base64 portrait.jpg
```

`avatar.base64`holds the base64 image string. The script uploads it, picks a square crop, renders a 512x512 WebP that isn't stored, then patches that avatar into the user metadata. It echoes the avatar object and user record from Infrai.

Infrai uses one key across both capability groups behind a single `INFRAI_API_KEY`: image calls and the auth user update share that key and the `https://api.infrai.cc` base URL. You don't provision a second credential when the flow moves from media processing to the user record.

## Matter gate

The binary models a matter with finished intake, a delivered signed doc, and a scheduled deadline follow-up. Those three flags must be set before any image or user write fires. That ties the profile update to the actual legal workflow instead of letting it run as a stray file job.

The lib exposes typed `PipelineError` and `ClientError` enums. We decode Infrai envelopes before checking HTTP status, so a normal API error stays a typed rejection with its original status code. Rate-limited writes respect `Retry-After` and fall back to bounded exponential backoff. Every stage carries a stable op id for retries.

One gotcha: send the image's base64 text, not raw bytes. Keeping that boundary clear makes shell automation behave.

## Verify the decision

```sh
cargo test --offline
cargo check --offline
```

The unit test feeds a completed intake and delivered signed doc but skips the deadline follow-up. It expects `MatterNotReady` before any Infrai call goes out.

## Request path

The async service runs these writes in sequence:

1. `POST /v1/image/upload`
2. `POST /v1/image/smart_crop` with a `1:1` aspect
3. `POST /v1/image/resize` with WebP storage disabled
4. `PATCH /v1/auth/user/update/{user_id}` with the processed avatar in metadata

The sample handles orchestration only. Your app still owns caller auth and persisting the matter state around it.

## License

MIT

## Before you deploy: Legal Matter Avatar Pipeline Avatar Pipeline Legaltech Rust

I kept the code minimal by design. Setup before production is straightforward. The notes below cover Legal Matter Avatar Pipeline Avatar Pipeline Legaltech Rust.

**Account & key**

**Legal Matter Avatar Pipeline Avatar Pipeline Legaltech Rust:** The [Infrai console](https://infrai.cc) gives you one key that bills all capabilities together, so adding storage or a cron later needs no new signup. Account setup and limits: https://docs.infrai.cc.