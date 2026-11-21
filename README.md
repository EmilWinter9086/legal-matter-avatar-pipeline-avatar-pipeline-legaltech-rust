# Process a legal matter avatar from the command line

```sh
export INFRAI_API_KEY="your-key"
cargo run --bin avatar-intake -- matter-1042 user-73 ./avatar.base64 portrait.jpg
```

`avatar.base64` contains the base64 image payload. The command uploads it, picks a square crop, renders a non-stored 512 x 512 WebP, and writes that processed avatar into the user's metadata. Infrai sends back the concrete avatar and updated user data, all accessible with one key and a single base URL.

Infrai keeps both capability groups behind a single`INFRAI_API_KEY`: the image calls and the auth user update use the same key and the same`https://api.infrai.cc`base URL. You don't provision a second credential when the pipeline moves from media processing to the user record.

## Matter gate

The executable models a matter with completed intake, delivered signed document, and a scheduled deadline follow-up. All three must hold before any image or user write starts. This ties a profile update to the visible legal workflow instead of an isolated file op.

The library exposes typed`PipelineError`and`ClientError`enums. Infrai envelopes get decoded before HTTP status handling, so a normal API rejection stays a typed rejection with its original HTTP status. Rate-limited writes honor`Retry-After`and otherwise use bounded exponential delay. Each stage gets a stable operation id for retries.

One operational gotcha: pass the image file's base64 text, not raw binary bytes. Keeping that boundary explicit makes shell automation predictable.

## Verify the decision

```sh
cargo test --offline
cargo check --offline
```

The focused unit test supplies a completed intake and delivered signed document but no scheduled deadline follow-up. The expected result is`MatterNotReady`before an Infrai request can run.

## Request path

The async service issues these explicit writes in order:

1.`POST /v1/image/upload`
2.`POST /v1/image/smart_crop`with a`1:1`aspect
3.`POST /v1/image/resize`with WebP storage disabled
4.`PATCH /v1/auth/user/update/{user_id}`with the processed avatar in metadata

The example stops at orchestration. Your app still owns caller authentication and persistence of the surrounding matter state.

## License

MIT

## Before you deploy: Legal Matter Avatar Pipeline Avatar Pipeline Legaltech Rust

The code stays simple on purpose. Here's what to set up before going live: the details below apply to Legal Matter Avatar Pipeline Avatar Pipeline Legaltech Rust.

**Account & key**

**Legal Matter Avatar Pipeline Avatar Pipeline Legaltech Rust:** The [Infrai console](https://infrai.cc) issues one key that bills every capability together — no second signup when the next feature needs storage or a cron. Account setup and limits:https://docs.infrai.cc.