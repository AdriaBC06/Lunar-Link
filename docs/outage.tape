# Captura de un corte de señal (espera a la cara oculta, ~85 s):
#   cargo build --release && vhs docs/outage.tape
Output "/tmp/lunar-link-outage.gif"

Set Shell "bash"
Set FontSize 14
Set Width 1000
Set Height 620
Set Padding 12
Set Theme "Catppuccin Mocha"

Hide
Type "./target/release/lunar-link --delay 1"
Enter
Sleep 1s
Show
Down 4
Enter
Sleep 80s
Enter
Sleep 3s
Screenshot docs/screenshot-outage.png
Hide
Type "q"
