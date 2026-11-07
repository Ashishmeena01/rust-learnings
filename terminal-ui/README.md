# Terminal Calendar

An interactive calendar built with Ratatui. Run it from this directory with:

```sh
cargo run
```

The app opens in a year overview. Use `?` to open the in-app shortcut guide and `q` to quit.

## Navigation

Move by day with `h`/`l` or the left/right arrows. Move by week with `j`/`k` or the up/down arrows. Use `n` and `p` for adjacent months, `y` and `Y` for adjacent years, and `t` to return to today.

## Views and themes

Press `v` or Enter to switch between the twelve-month overview and the focused month view. Press `c` to cycle through B&W Obsidian, B&W Paper, Neobrutalist B&W, and Tokyo Night themes. Press `s` to cycle the calendar's date-label style.

## Events

The calendar highlights dates from a built-in annual observance list. The focused month view lists that month's events, while the inspector shows the selected date's event (if any) and the next upcoming event countdown.

## Date inspector

The inspector summarizes the selected date, its season and quarter, and its day-of-year progress. The `t` shortcut resets the selection to the current local date when the system clock is available.

## Help and exit

Press `?` to open the shortcut overlay. Any key dismisses it; `q` exits the application directly from the overlay. Outside the overlay, `q` or Escape exits.
