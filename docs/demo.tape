# Genera docs/demo.gif ejecutando: vhs docs/demo.tape
# Requiere vhs (https://github.com/charmbracelet/vhs) y el binario compilado.
Output docs/demo.gif

Set Shell "bash"
Set FontSize 16
Set Width 1000
Set Height 600
Set Theme "Catppuccin Mocha"
Set Padding 12
Set TypingSpeed 60ms

Type "cargo run --release"
Enter
Sleep 2s

# Menú principal -> Ping
Down 5
Sleep 400ms
Enter
Sleep 4s

# Volver y descargar un archivo
Escape
Up 4
Enter
Sleep 600ms
Enter
Sleep 6s

Type "q"
Sleep 500ms
