# Vendored `google/api` Protos

The `.proto` files in this directory are vendored from [`googleapis/googleapis`](https://github.com/googleapis/googleapis/tree/master/google/api) so downstream crates depending on `google-protobuf-roto` have standard HTTP and [AIP](https://google.aip.dev) annotation options on their `protoc` include path (`-I <google-protobuf-roto>/proto`) and generated `::google_protobuf_roto::google::api` Rust views/builders.

- **Source repository**: `https://github.com/googleapis/googleapis`
- **Commit SHA**: `56eddd2a45213efec86ad4a36246923b0b7a1a58`
- **License**: Apache License, Version 2.0 (`Apache-2.0`, see headers in each file and `http://www.apache.org/licenses/LICENSE-2.0`), Copyright Google LLC.

## Files

- `annotations.proto` & `http.proto` (`google.api.http`, `Http`, `HttpRule`, `CustomHttpPattern`)
- `client.proto` (`google.api.method_signature`, `google.api.default_host`, `google.api.oauth_scopes`, `google.api.api_version`, `ClientLibrarySettings`, `MethodSettings`, etc.)
- `field_behavior.proto` (`google.api.field_behavior`, `FieldBehavior`, AIP-203)
- `launch_stage.proto` (`LaunchStage`, imported by `client.proto`)
- `resource.proto` (`google.api.resource`, `google.api.resource_reference`, `google.api.resource_definition`, `ResourceDescriptor`, `ResourceReference`, AIP-123)
