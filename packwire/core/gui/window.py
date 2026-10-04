import sys
import webbrowser
import time
from typing import Optional

from packwire.core.gui.bridge import GuiBridge
from packwire.core.gui.server import run_local_server


def _hide_console_window():
    """Hide the background console window on Windows if spawned exclusively for this process."""
    if sys.platform == "win32":
        try:
            import ctypes
            kernel32 = ctypes.windll.kernel32
            user32 = ctypes.windll.user32
            hwnd = kernel32.GetConsoleWindow()
            if hwnd:
                pids = (ctypes.c_uint * 2)()
                count = kernel32.GetConsoleProcessList(pids, 2)
                # If only 1 process is attached, console was created for this GUI window
                if count <= 1:
                    user32.ShowWindow(hwnd, 0)  # SW_HIDE
        except Exception:
            pass


def launch_gui(web_mode: bool = False, port: int = 5050):
    """
    Launch the Packwire User Interface.
    - If web_mode=False and pywebview is installed: Opens native desktop window.
    - If web_mode=True or pywebview is missing: Starts local server and opens browser.
    """
    _hide_console_window()
    bridge = GuiBridge()
    server, url = run_local_server(port=port)

    has_webview = False
    if not web_mode:
        try:
            import webview
            has_webview = True
        except ImportError:
            has_webview = False

    gui_opened = False
    if has_webview and not web_mode:
        try:
            import webview
            print("[*] Iniciando ventana de escritorio nativa (Edge/WebKit)...")
            window = webview.create_window(
                title="Packwire",
                url=url,
                js_api=bridge,
                width=1340,
                height=840,
                min_size=(1040, 680)
            )
            webview.start()
            print("[*] Ventana cerrada.")
            gui_opened = True
        except Exception as e:
            print(f"[!] No se pudo inicializar la ventana nativa ({e}). Abriendo en navegador web...")
            gui_opened = False

    if not gui_opened:
        print(f"[*] Servidor local activo en: {url}")
        print("[*] Abriendo interfaz en tu navegador predeterminado...")
        webbrowser.open(url)
        print("[*] Presiona Ctrl+C en esta terminal para detener el servidor.")
        try:
            while True:
                time.sleep(1)
        except KeyboardInterrupt:
            print("\n[*] Servidor detenido.")
