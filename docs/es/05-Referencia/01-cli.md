# Interfaz de Línea de Comandos

La CLI nativa está implementada en `packwire-rs/src/cli.rs` y se compila como `packwire.exe`.

## Comandos

```text
packwire install <paquete> [--version <versión>] [--channel stable|fixed] [--mode here|site|command] [--force]
packwire list
packwire available
packwire info <paquete>
packwire path [--add]
packwire --help
packwire --version
```

Ejemplos:

```text
packwire available
packwire install python@stable --mode here
packwire install rust --channel stable
packwire list
packwire path --add
```

La GUI se ejecuta con el binario separado `packwirew.exe`, que no crea una ventana de consola.
