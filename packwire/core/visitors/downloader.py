import urllib.request
import time
import sys
from pathlib import Path
from typing import Optional, Callable
import hashlib

from packwire.core.config import CACHE_DIR, ensure_directories


class DownloaderVisitor:
    """Visitor responsible for downloading package assets with progress and caching."""

    def __init__(self, cache_dir: Optional[Path] = None):
        self.cache_dir = cache_dir or CACHE_DIR
        ensure_directories()

    def download(
        self,
        url: str,
        dest_filename: Optional[str] = None,
        expected_hash: Optional[str] = None,
        progress_callback: Optional[Callable[[int, int, float], None]] = None,
        force: bool = False
    ) -> Path:
        """
        Download a file from URL to the cache directory.
        progress_callback signature: (downloaded_bytes, total_bytes, speed_bps)
        """
        if not dest_filename:
            # Infer filename from URL
            dest_filename = url.split("?")[0].rstrip("/").split("/")[-1]
            if not dest_filename:
                dest_filename = "download.bin"

        dest_path = self.cache_dir / dest_filename

        # If file already exists and not forced, check hash
        if dest_path.exists() and not force:
            if expected_hash:
                actual_hash = self._calc_sha256(dest_path)
                if actual_hash.lower() == expected_hash.lower():
                    print(f"[Packwire Downloader] Usando archivo en caché: {dest_path.name}")
                    return dest_path
            else:
                # If file exists and has size > 0, we can reuse it
                if dest_path.stat().st_size > 0:
                    print(f"[Packwire Downloader] Usando archivo existente en caché: {dest_path.name}")
                    return dest_path

        part_path = dest_path.with_suffix(dest_path.suffix + ".part")

        print(f"[Packwire Downloader] Descargando desde: {url}")
        req = urllib.request.Request(
            url,
            headers={"User-Agent": "Packwire/0.1.0 (Windows; Python)"}
        )

        try:
            with urllib.request.urlopen(req) as response:
                total_size = int(response.getheader("Content-Length", 0))
                downloaded = 0
                block_size = 64 * 1024  # 64 KB chunks
                start_time = time.time()
                last_print = 0

                with open(part_path, "wb") as out_file:
                    while True:
                        chunk = response.read(block_size)
                        if not chunk:
                            break
                        out_file.write(chunk)
                        downloaded += len(chunk)

                        now = time.time()
                        elapsed = now - start_time
                        speed = downloaded / elapsed if elapsed > 0 else 0

                        if progress_callback:
                            progress_callback(downloaded, total_size, speed)
                        elif now - last_print > 0.15 or downloaded == total_size:
                            self._print_cli_progress(downloaded, total_size, speed)
                            last_print = now

            if total_size > 0 and downloaded < total_size:
                part_path.unlink(missing_ok=True)
                raise IOError(f"Descarga incompleta: recibidos {downloaded} de {total_size} bytes.")

            # Atomically rename .part to dest_path
            part_path.replace(dest_path)

        except Exception:
            part_path.unlink(missing_ok=True)
            raise

        print()  # Newline after progress bar
        print(f"[Packwire Downloader] Descarga completada: {dest_path.name}")

        if expected_hash:
            actual_hash = self._calc_sha256(dest_path)
            if actual_hash.lower() != expected_hash.lower():
                dest_path.unlink(missing_ok=True)
                raise ValueError(
                    f"Error de integridad: El hash esperado ({expected_hash}) no coincide con el descargado ({actual_hash})"
                )

        return dest_path

    def _calc_sha256(self, filepath: Path) -> str:
        sha = hashlib.sha256()
        with open(filepath, "rb") as f:
            while chunk := f.read(128 * 1024):
                sha.update(chunk)
        return sha.hexdigest()

    def _print_cli_progress(self, downloaded: int, total: int, speed: float):
        mb_down = downloaded / (1024 * 1024)
        speed_mb = speed / (1024 * 1024)
        if total > 0:
            mb_total = total / (1024 * 1024)
            pct = (downloaded / total) * 100
            bar_len = 30
            filled = int(bar_len * downloaded / total)
            bar = "#" * filled + "-" * (bar_len - filled)
            sys.stdout.write(
                f"\r[{bar}] {pct:5.1f}% ({mb_down:6.2f}MB / {mb_total:6.2f}MB) @ {speed_mb:5.2f} MB/s"
            )
        else:
            sys.stdout.write(f"\rDescargando: {mb_down:6.2f}MB @ {speed_mb:5.2f} MB/s")
        sys.stdout.flush()
