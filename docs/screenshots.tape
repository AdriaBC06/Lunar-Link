# Genera las capturas de docs/:  cargo build --release && vhs docs/screenshots.tape
Output /tmp/lunar-link-shots.gif

Set Shell "bash"
Set FontSize 14
Set Width 1000
Set Height 620
Set Padding 12
Set Theme "Catppuccin Mocha"

# Modo Luna: telemetría en vivo.
Hide
Type "./target/release/lunar-link --no-outages"
Enter
Sleep 1s
Show
Down 2
Enter
Sleep 6s
Screenshot docs/screenshot-luna.png
Hide
Type "q"
Sleep 500ms

# Modo Marte: dos peticiones en vuelo con el tiempo acelerado.
Type "./target/release/lunar-link --target marte --speed 100 --no-outages"
Enter
Sleep 1s
Show
Enter
Escape
Down
Enter
Sleep 4s
Screenshot docs/screenshot-marte.png
Hide
Type "q"
