# Genera docs/demo.gif:  cargo build --release && vhs docs/demo.tape
# Requiere vhs (https://github.com/charmbracelet/vhs), ttyd y ffmpeg.
Output docs/demo.gif

Set Shell "bash"
Set FontSize 14
Set Width 1000
Set Height 620
Set Padding 12
Set Theme "Catppuccin Mocha"
Set TypingSpeed 40ms

Hide
Type "./target/release/lunar-link --no-outages"
Enter
Sleep 1s
Show
Sleep 1s

# Ping: el paquete viaja a la Luna y vuelve.
Down 4
Enter
Enter
Sleep 3.2s

# Archivos: lista y descarga con barra de progreso.
Escape
Up 3
Enter
Sleep 3s
Enter
Sleep 5s

Hide
Escape
Type "q"
