# Captura de la galería ASCII:  cargo build --release && vhs docs/screenshot-galeria.tape
Output "/tmp/lunar-link-galeria.gif"

Set Shell "bash"
Set FontSize 14
Set Width 1000
Set Height 620
Set Padding 12
Set Theme "Catppuccin Mocha"

Hide
Type "./target/release/lunar-link --no-outages"
Enter
Sleep 2s
Show
Down 5
Enter
Sleep 3.5s
Right
Sleep 3.5s
Screenshot docs/screenshot-galeria.png
Hide
Type "q"
