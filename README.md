# tw-econ

## Description

Rust library provides you a simple synchronous interface to interconnect with Teeworlds external console.

## Example

Let's say you have Teeworlds server running with `ec_password zohan` and `ec_port 6060` and you want to use it's econ.

```rust
use tw_econ::Econ;

fn main() -> std::io::Result<()> {
    let mut econ = Econ::new();

    econ.connect("127.0.0.1:6060")?;

    let authed = econ.try_auth("nahoz")?;
    assert_eq!(authed, false);

    let authed = econ.try_auth("hozan")?;
    assert_eq!(authed, false);

    let authed = econ.try_auth("zohan")?;
    assert_eq!(authed, true);

    econ.send_line("echo \"Hi\"")?;

    econ.fetch()?;
    
    println!("{}", econ.pop_line()?);

    Ok(())
}
```

## Projects

* [tw-econ-tui](https://github.com/gerdoe-jr/tw-econ-tui)
