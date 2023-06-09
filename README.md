# FastConnect

FastConnect is a cross-platform CLI based tool to connect to Binus-Access, that is written in Rust.

## How to build

### Cargo

```bash
cargo build --release
```

### build.sh

#### Cross compile

- x86_64-unknown-linux-gnu
- x86_64-unknown-linux-gnux32
- x86_64-unknown-linux-musl
- i686-unknown-linux-gnu
- i686-unknown-linux-musl
- i586-unknown-linux-gnu
- i586-unknown-linux-musl
- aarch64-unknown-linux-gnu
- aarch64-unknown-linux-musl
- x86_64-pc-windows-gnu
- i686-pc-windows-gnu
- x86_64-apple-darwin
- aarch64-apple-darwin

#### Dependencies

- [rustup](https://rustup.rs/)
- [zig](https://ziglang.org/)
- [zigbuild](https://github.com/rust-cross/cargo-zigbuild)
- [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
- [MacOS SDK](https://developer.apple.com/macos/)

#### Running the script

```bash
export ZIG_HOME=/path/to/zig
export MACOS_SDK=/path/to/MACOS11_3_SDK # I can't include the SDK in the repo due to the licensing
bash build.sh
```

## Logic

### Backaccess
FastConnect uses the following API to connect to Binus-Access:

> https://backaccess.apps.binus.edu/wifi/loginValidator.php?=&prop=revisions&rvprop=content&format=json&callback=?&origin=*

This API is used to get the id of the user, that will be used to connect to Binus-Access. To get the user ID we need
to send a form like this
```rust
let request_form =  [
    ("username", format!("{}@{}", "EMAIL_NAME", "email_tld")),
    ("password", format!("{}", "user_password"))
];
```
---
### Login

> https://access.apps.binus.ac.id/login

After getting the userID, FastConnect will use this API to connect to Binus-Access. It will send
a form like this
```rust
let request_form =  [
    ("dst", ""),
    ("popup", "false"),
    ("username", "userID"),
    ("password", "User Password")
];
```

Then it will check if the response contains following string
```text
You are about to access the Internet Service operated by BINUS.
```
This string is used to check if the user is connected to Binus-Access. If the response
does not contain this string, it means that you're successfully connected to Binus-Access.