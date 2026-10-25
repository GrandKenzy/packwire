# Plan de datos del sistema

La pestaña **Sistema** debe funcionar como una ficha técnica local para elegir
toolchains, comprobar compatibilidad y explicar problemas de instalación. Se
consulta bajo demanda y no se envía a ningún servidor.

## Datos que sí se muestran

| Grupo | Datos | Utilidad práctica |
| :--- | :--- | :--- |
| Windows | Edición, DisplayVersion, versión, build y arquitectura | Saber si un paquete requiere Windows 10/11, x64 o ARM64. |
| Equipo | Fabricante, modelo y RAM total/libre | Detectar equipos antiguos, poca memoria o virtualización. |
| CPU | Modelo, núcleos, hilos, frecuencia máxima y virtualización de firmware | Elegir toolchains, interpretar rendimiento y comprobar soporte de máquinas virtuales. |
| GPU | Modelo, memoria aproximada y versión del driver | Diagnosticar compilación gráfica, CUDA/OpenCL y drivers desactualizados. |
| Almacenamiento | Volúmenes locales, sistema de archivos, tamaño y espacio libre | Explicar fallos por poco espacio y decidir dónde guardar SDKs grandes. |
| Firmware | Modo UEFI/confirmación, fabricante, versión y fecha del BIOS | Detectar firmware antiguo y anticipar requisitos de Rufus, Ventoy o Linux. |
| Seguridad | Secure Boot, presencia/estado de TPM y versión de especificación | Explicar requisitos de Windows 11 y por qué algunas imágenes Linux piden ajustes de arranque. |
| Ejecución | Tiempo desde el último arranque y presencia de hipervisor | Detectar reinicios pendientes, WSL/Hyper-V y diferencias de rendimiento. |
| Red | Solo nombre y velocidad de adaptadores físicos activos | Saber si una descarga lenta está limitada por Wi-Fi/Ethernet sin revelar la red local. |

## Datos que se excluyen

No se recopilan números de serie, claves de producto, nombre de usuario, MAC,
IP, SSID, historial de navegación, procesos, archivos, contenido de discos ni
la lista de programas instalados fuera de Packwire. El diagnóstico tampoco
modifica BIOS, Secure Boot, Defender, TPM, PATH ni configuración de red.

## Criterios de implementación

1. Usar consultas de solo lectura del sistema local y mostrar `No disponible`
   cuando una edición de Windows no ofrezca un dato.
2. Mantener los valores en memoria de la interfaz; no crear un inventario
   persistente ni subir telemetría.
3. Evitar conclusiones tajantes: el perfil de Linux o toolchain recomendado es
   orientativo y debe contrastarse con los requisitos del proyecto.
4. Revisar las sumas SHA-256 o firmas publicadas por el proveedor antes de
   ejecutar imágenes o binarios descargados.
