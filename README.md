# harpoon-rs

Harpoon-rs allows you to bind application windows to a quick select menu.
Windows added to harpoon can be managed in the menu which can be opened with **(L)Ctrl + (L)Alt + H**.
The top eight windows are focusable with a dedicated keybind.

## Known issues:

- Harpoon stops responding after locking my device with `WIN + L` or using `CTRL + ALT + DEL`/`CTRL SHIFT ESC`
  To fix this issue, simply reactivate those buttons individually.
  Windows takes over keyboard input the moment these shortcuts are hit, causing harpoon to not realise they're no longer pressed. Simply hitting them again fixes this.

## Supported platforms

- Windows only for now

## Installation

```sh
cargo install --git https://github.com/Dutch-Raptor/harpoon-rs --locked harpoon-rs
```

_Or clone the repository and use `cargo install --path .`_

## Administrator privilage

Administrator privilage is only required if you want to add a program to harpoon that is running as administrator.

## Keybinds

These are all the default keybinds that can be used to interact with harpoon.

### Key binds when not in the menu 

|keys   |function   |
|:---   |:---   |
| (L)Ctrl + (L)Alt + H | Toggle quick menu |
| (L)Ctrl + (L)Alt + A | Add current window |
| (L)Ctrl + (L)Alt + M | Navigate to next window |
| (L)Ctrl + (L)Alt + N | Navigate to previous window |
| (L)Ctrl + (L)Alt + S | Toggle Inhibit |
| (L)Ctrl + (L)Alt + J | Focus on window 1 |
| (L)Ctrl + (L)Alt + K | Focus on Window 2 |
| (L)Ctrl + (L)Alt + L | Focus on window 3 |
| (L)Ctrl + (L)Alt + ; | Focus on window 4 |
| (L)Ctrl + (L)Alt + U | Focus on window 5 |
| (L)Ctrl + (L)Alt + I | Focus on window 6 |
| (L)Ctrl + (L)Alt + O | Focus on window 7 |
| (L)Ctrl + (L)Alt + P | Focus on window 8 |

### key binds when in the menu

|function   | Keys  |
|:---   |:---   |
| Quit | Q, Esc |
| Confirm selection | Enter, Space |
| Move selection down | &darr;, J |
| Move selection up | &uarr;, K |
| Swap down | (L)Alt + &darr;, (L)Alt + J |
| Swap up | (L)Alt + &uarr;, (L)Alt + K |
| Cut | Backspace, (L)Shift + D |
| Paste Down | P |
| Paste Up | (L)Shift + P |
