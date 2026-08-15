# utui

`utui` is a small immediate-mode terminal UI library built on
[`crossterm`](https://crates.io/crates/crossterm). It composes state-driven rendering callbacks
with text, vertical stacks, and scrollable lists while leaving terminal setup and input handling to
the application.

```rust
use std::{error::Error, io::{stdout, Stdout}};
use utui::{Area, Context, Text, Tui};

fn main() -> Result<(), Box<dyn Error>> {
    let view = |ctx: &mut Context<'_, String, Stdout>| ctx.line(&|text| Ok(text.clone()));
    let tui = Tui::new(&view);
    tui.draw(&mut stdout(), &Area::from((80, 24)), &"Hello, terminal!".to_owned())
}
```

## Licensing

Dual-licensed under the **MIT** and **Apache License 2.0**. See the included
`LICENSE` file.
