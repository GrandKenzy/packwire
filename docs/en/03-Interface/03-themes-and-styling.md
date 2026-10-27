# Themes & Styling

Packwire uses a modular CSS architecture that enables real-time switching between color schemes without reloading the DOM.

---

## 1. Modular CSS Layout

Styles inside `assets/web/global/` are split into functional domains:
- **`structure.css`:** Core grid, layout dimensions, flex containers, modal overlays, and typography.
- **`*_background.css`:** Background colors, surface layers, and scrollbar styles.
- **`*_cards.css`:** Card borders, elevations, hover states, and badge colors.
- **`*_buttons.css`:** Action button gradients, outlines, and interactive focus states.

---

## 2. Preference Persistence

The active theme is persisted in two locations:
1. **Frontend `localStorage`:** Key `packwire_theme` ensures instantaneous CSS application before DOM rendering to prevent flashing.
2. **Backend `configs.json`:** Synced via `bridge.set_config("theme", name)` so settings persist across reboots and CLI sessions.
