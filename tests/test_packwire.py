import unittest
import tempfile
import shutil
from pathlib import Path

from packwire.core.models import Manifest, InstallConfig, HereConfig
from packwire.core.state import check_collision, register_installed, unregister_installed, load_state, PackageState
from packwire.core.visitors.process_manifest import ManifestVisitor
from packwire.core.visitors.pather import PatherVisitor
from packwire.core.resolver import PythonResolver, resolve_manifest


class TestPackwire(unittest.TestCase):

    def test_canonical_hash(self):
        m1 = Manifest(
            id="python@3.14",
            name="Python",
            type="fixed",
            version="3.14.8",
            icon="🐍",
            install=InstallConfig(mode="here", clean=True)
        )
        h1 = m1.calculate_hash()
        self.assertTrue(len(h1) == 64)
        # Idempotence
        self.assertEqual(h1, m1.calculate_hash())

    def test_manifest_icon(self):
        m_emoji = Manifest.from_dict({"id": "pkg1", "name": "Pkg1", "icon": "⚡"})
        self.assertEqual(m_emoji.icon, "⚡")

        m_url = Manifest.from_dict({"id": "pkg2", "name": "Pkg2", "icon": "https://example.com/icon.svg"})
        self.assertEqual(m_url.icon, "https://example.com/icon.svg")

        m_default = Manifest.from_dict({"id": "pkg3", "name": "Pkg3"})
        self.assertEqual(m_default.icon, "📦")

    def test_python_resolver_stable(self):
        version, dl_url, site_url = PythonResolver.resolve_latest_stable()
        self.assertTrue(version.startswith("3."))
        self.assertIn("python.org", dl_url)
        self.assertIn("python.org", site_url)

    def test_python_resolver_fixed(self):
        version, dl_url, site_url = PythonResolver.resolve_fixed("3.12.8")
        self.assertEqual(version, "3.12.8")
        self.assertEqual(dl_url, "https://www.python.org/ftp/python/3.12.8/python-3.12.8-embed-amd64.zip")

    def test_pather_shims(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = Path(temp_dir)
            shims_dir = temp_path / "shims"
            bin_dir = temp_path / "bin"
            shims_dir.mkdir()
            bin_dir.mkdir()

            dummy_exe = bin_dir / "python.exe"
            dummy_exe.write_text("fake binary")

            pather = PatherVisitor(shims_dir=shims_dir)
            created = pather.create_shims(bin_dir, ["python.exe"], version_suffix="3.14.0", is_default=True)

            self.assertIn("python.cmd", created)
            self.assertTrue((shims_dir / "python.cmd").exists())
            self.assertTrue((shims_dir / "python3140.cmd").exists())

            pather.remove_shims(created)
            self.assertFalse((shims_dir / "python.cmd").exists())

    def test_check_updates_and_reinstall(self):
        import packwire
        # Test check_updates returns dictionary
        updates = packwire.check_updates()
        self.assertIsInstance(updates, dict)

        # Test reinstall non-existent package fails gracefully
        ok, msg = packwire.reinstall("non_existent_pkg")
        self.assertFalse(ok)

    def test_config_persistence(self):
        import packwire
        from packwire.core.gui.bridge import GuiBridge

        bridge = GuiBridge()

        # Test setting and getting theme
        res = bridge.set_config("theme", "dark")
        self.assertTrue(res.get("success"))

        theme = bridge.get_config("theme")
        self.assertEqual(theme, "dark")

        # Full config dict
        all_cfg = bridge.get_config()
        self.assertIsInstance(all_cfg, dict)
        self.assertEqual(all_cfg.get("theme"), "dark")

        # Restore default
        bridge.set_config("theme", "default")
        self.assertEqual(bridge.get_config("theme"), "default")

    def test_new_manifests_and_aliases(self):
        import packwire
        from packwire.core.resolver import NodeResolver

        # Test aliases for newly added packages
        c_manifest = packwire.get_manifest("c")
        self.assertIsNotNone(c_manifest)
        self.assertEqual(c_manifest["id"], "c-gcc")
        self.assertIn("gcc.exe", c_manifest["install"]["here"]["binaries"])

        ffmpeg_manifest = packwire.get_manifest("ffmpeg")
        self.assertIsNotNone(ffmpeg_manifest)
        self.assertEqual(ffmpeg_manifest["id"], "ffmpeg@stable")
        self.assertIn("ffmpeg.exe", ffmpeg_manifest["install"]["here"]["binaries"])

        node_manifest = packwire.get_manifest("node")
        self.assertIsNotNone(node_manifest)
        self.assertEqual(node_manifest["id"], "nodejs@stable")
        self.assertIn("node.exe", node_manifest["install"]["here"]["binaries"])

        php_manifest = packwire.get_manifest("php")
        self.assertIsNotNone(php_manifest)
        self.assertEqual(php_manifest["id"], "php@stable")
        self.assertIn("php.exe", php_manifest["install"]["here"]["binaries"])

        vscode_manifest = packwire.get_manifest("vscode")
        self.assertIsNotNone(vscode_manifest)
        self.assertEqual(vscode_manifest["id"], "vscode")
        self.assertEqual(vscode_manifest["install"]["mode"], "site")

        # Test NodeResolver
        ver, dl, site = NodeResolver.resolve_latest_lts()
        self.assertTrue(len(ver) > 0)
        self.assertTrue(dl.endswith(".zip"))

    def test_additional_manifests(self):
        import packwire

        # Test chocolatey
        choco = packwire.get_manifest("choco")
        self.assertIsNotNone(choco)
        self.assertEqual(choco["install"]["mode"], "command")
        self.assertTrue(choco["install"]["command"]["elevated"])

        # Test msys2 and pacman (separate cards)
        msys2 = packwire.get_manifest("msys2")
        self.assertIsNotNone(msys2)
        self.assertEqual(msys2["id"], "msys2")

        pacman = packwire.get_manifest("pacman")
        self.assertIsNotNone(pacman)
        self.assertEqual(pacman["id"], "pacman")

        # Test ffprobe
        ffprobe = packwire.get_manifest("ffprobe")
        self.assertIsNotNone(ffprobe)
        self.assertEqual(ffprobe["id"], "ffprobe")

        # Test sdl3
        sdl3 = packwire.get_manifest("sdl3")
        self.assertIsNotNone(sdl3)
        self.assertEqual(sdl3["id"], "sdl3")

        # Test rust
        rust = packwire.get_manifest("rust")
        self.assertIsNotNone(rust)
        self.assertEqual(rust["install"]["mode"], "command")

        # Test csharp
        csharp = packwire.get_manifest("csharp")
        self.assertIsNotNone(csharp)
        self.assertEqual(csharp["install"]["mode"], "command")

        # Test git
        git_pkg = packwire.get_manifest("git")
        self.assertIsNotNone(git_pkg)
        self.assertEqual(git_pkg["id"], "git")
        self.assertIn("git", git_pkg["aliases"])
        self.assertEqual(git_pkg["install"]["mode"], "command")

        # Test docker
        docker_pkg = packwire.get_manifest("docker")
        self.assertIsNotNone(docker_pkg)
        self.assertEqual(docker_pkg["id"], "docker")
        self.assertIn("docker-desktop", docker_pkg["aliases"])
        self.assertTrue(docker_pkg["install"]["command"]["elevated"])

    def test_task_queue(self):
        import packwire
        from packwire.core.task_queue import task_queue

        # Test submitting a mock/test job
        job_id = task_queue.submit_install("non_existent_pkg")
        self.assertTrue(job_id.startswith("job-"))

        # Poll job
        import time
        time.sleep(0.5)
        job = task_queue.get_job(job_id)
        self.assertIsNotNone(job)
        self.assertIn(job["status"], ["queued", "running", "failed", "completed"])
        self.assertIsInstance(job["logs"], list)

    def test_uninstall_all(self):
        import packwire
        from packwire.core.state import register_installed, load_state
        from packwire.core.models import PackageState
        from packwire.core.config import SHIMS_DIR

        # Create dummy package state and dummy shim
        dummy_state = PackageState(
            id="test-dummy@1.0",
            name="Test Dummy",
            version="1.0",
            type="fixed",
            mode="here",
            install_path=str(packwire.APPS_DIR / "test_dummy"),
            shims=["dummy_bin.cmd"]
        )
        register_installed(dummy_state)
        (SHIMS_DIR / "dummy_bin.cmd").write_text("@echo off", encoding="utf-8")

        self.assertIn("test-dummy@1.0", load_state())
        self.assertTrue((SHIMS_DIR / "dummy_bin.cmd").exists())

        # Run uninstall_all
        ok, msg = packwire.uninstall_all(remove_path=False)
        self.assertTrue(ok)
        self.assertIn("Desinstalación completa", msg)

        # State should now be empty and shim gone
        self.assertEqual(len(load_state()), 0)
        self.assertFalse((SHIMS_DIR / "dummy_bin.cmd").exists())

        # Test GuiBridge uninstall_all method
        from packwire.core.gui.bridge import GuiBridge
        bridge = GuiBridge()
        bridge_res = bridge.uninstall_all(remove_path=False)
        self.assertTrue(bridge_res["success"])

    def test_self_installer_and_packager(self):
        import packwire
        from packwire.packager import ensure_icon
        from packwire.installer_self import install_self
        from packwire.core.config import SHIMS_DIR

        # Test ensure_icon
        ico = ensure_icon()
        self.assertTrue(ico.exists())
        self.assertTrue(ico.stat().st_size > 0)

        # Test self-install shims creation (without modifying system PATH or shortcuts)
        ok, msg = install_self(add_to_path=False, create_start_menu=False, create_desktop=False)
        self.assertTrue(ok)
        self.assertTrue((SHIMS_DIR / "packwire.cmd").exists())
        self.assertTrue((SHIMS_DIR / "packwire.ps1").exists())

        cmd_content = (SHIMS_DIR / "packwire.cmd").read_text(encoding="utf-8")
        self.assertIn("packwire", cmd_content)


if __name__ == "__main__":
    unittest.main()
