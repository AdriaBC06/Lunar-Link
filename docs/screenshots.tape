# Captura del modo Luna (telemetría en vivo):  cargo build --release && vhs docs/screenshots.tape
Output "/tmp/lunar-link-luna.gif"

Set Shell "bash"
Set FontSize 14
Set Width 1000
Set Height 620
Set Padding 12
Set Theme "Catppuccin Mocha"

Hide
Type "./target/release/lunar-link --no-outages"
Enter
Sleep 1s
Show
Down 2
Enter
Sleep 25s
Screenshot docs/screenshot-luna.png
Hide
Type "q"
