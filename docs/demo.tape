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
Sleep 2s
Show
Sleep 2s

# Ping: el paquete viaja a la Luna y vuelve.
Down 4
Sleep 500ms
Enter
Sleep 300ms
Enter
Sleep 4.5s

# Archivos: lista y descarga con barra de progreso.
Escape
Up 3
Enter
Sleep 3.5s
Enter
Sleep 6.5s

Hide
Escape
Type "q"
