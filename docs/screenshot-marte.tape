# Captura del modo Marte con el tiempo acelerado:  cargo build --release && vhs docs/screenshot-marte.tape
Output "/tmp/lunar-link-marte.gif"

Set Shell "bash"
Set FontSize 14
Set Width 1000
Set Height 620
Set Padding 12
Set Theme "Catppuccin Mocha"

Hide
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
