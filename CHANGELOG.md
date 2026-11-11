# Changelog — GLASTOR® Strata

<p align="center">
  <img src="https://img.shields.io/badge/Format-Keep%20a%20Changelog-blue.svg" alt="Keep a Changelog">
  <img src="https://img.shields.io/badge/Versioning-SemVer%202.0.0-green.svg" alt="Semantic Versioning">
  <img src="https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg" alt="License">
  <img src="https://img.shields.io/badge/Repository-glastor--core%2Fstrata-informational" alt="Repository">
</p>

---

## Acerca de este Changelog

Todos los cambios notables en **GLASTOR® Strata** (Zero-Node Headless Accessibility, Atomic CSS & UI Engine for Rust WebAssembly) son documentados en este archivo.

Este proyecto se adhiere a:
- **Formato:** [Keep a Changelog](https://keepachangelog.com/es-ES/1.0.0/)
- **Versionado:** [Semantic Versioning](https://semver.org/spec/v2.0.0.html)

---

## [0.1.0] - 2026-11-09

### ✨ Added

#### `glastor-aria-core`
- **DialogFsm & FocusTrap**: Atrapamiento determinista de foco con centinelas virtuales, restauración de foco previo y descarte en cascada por tecla Escape.
- **AccordionFsm**: Modo `Single` y `Multi` con roving tabindex por teclado y gestión estricta de `aria-expanded`.
- **PopoverFsm**: Posicionamiento anclado y activación mediante click/hover.
- **TabsFsm**: Navegación horizontal y vertical con flechas de teclado (Left/Right/Home/End), activación automática vs manual y descarte de tabs deshabilitadas.
- **ToastFsm**: Notificaciones en regiones activas W3C (`aria-live="polite"` y `"assertive"`), cola de prioridad, pausa en hover/foco y descarte gestual.
- **VirtualGridFsm**: Grilla de datos 2D con navegación roving bidimensional, soporte para 100,000+ filas mediante slicing virtual (`VirtualSlice`) y atributos `role="grid"` / `role="gridcell"`.
- **ReactiveFsm**: Adaptador reactivo universal para señales de **Leptos** (`RwSignal`), **Dioxus** (`Signal`) o celdas en memoria.
- **Hardening iOS**: Emisión de acciones `LockBodyScroll` y `UnlockBodyScroll` para prevenir el rebote elástico en móviles.

#### `glastor-css-binder`
- **Zero-Node Compiler**: Escáner de clases utilitarias basado en `rayon` para procesar árboles fuente de Rust/RSX en microsegundos dentro de `build.rs`.
- **Motor de Variantes**: Soporte de pseudo-clases `hover:`, `focus:`, `focus-visible:`, `active:`, `disabled:` y modo oscuro (`dark:`).
- **Valores Arbitrarios**: Resolución y sanitización de utilidades arbitrarias (`w-[350px]`, `border-[#27272a]`) con protección contra inyecciones CSS y exfiltración de datos.
- **glastor-motion**: Motor físico de osciladores armónicos amortiguados (*spring physics*) que calcula en tiempo de build `@keyframes` CSS optimizados para ejecución en el GPU a 120 FPS (`animate-spring-in`, `animate-spring-bounce`, `animate-spring-slide`, `animate-spring-pop`).
- **Hot-Reloading Daemon**: Modo vigilancia (`CssWatcher` / `cargo ui watch`) con recompilación en <35ms.

#### `glastor-ui-cli` (`cargo ui` / `cargo strata`)
- **Distribución Descentralizada**: Comandos `cargo ui add <comp>` para inyección directa de código fuente con mantenimiento automático de `mod.rs`.
- **Multi-Framework CodeGen**: Parámetro `--framework leptos|dioxus|agnostic` para generar código adaptado al runtime del proyecto.
- **Catálogo Canónico (7 Componentes)**: `accordion`, `button`, `dialog`, `grid`, `popover`, `tabs` y `toast`.
- **W3C Design Tokens Sync**: Comando `cargo ui sync-tokens` para convertir archivos DTCG `tokens.json` de Figma/Penpot en variables CSS y constantes fuertemente tipadas en Rust.
- **Auditoría Estática WCAG**: Comando `cargo ui lint-a11y` para detectar violaciones de accesibilidad (tabindex positivo, labels faltantes en botones).
- **Integración GLASTOR® Core**: Comando `cargo ui audit` verificado con `specter` y `cargo ui generate-llms` para generación de `llms.txt` consumible por `zerho`.

#### Activos & Presentación
- **Favicon Suite Completa**: Formatos vectoriales SVG (`favicon.svg`, `byte-gauge.svg`, `specter.svg`, `zerho.svg`), bitmaps Retina (16, 32, 180, 192, 512), `favicon.ico` y `site.webmanifest` en `public/favicon/`.
- **Landing Page & Showcase**: Interfaz interactiva (`index.html`) con simuladores en vivo de compilación CSS, terminal CLI y demos de todas las FSMs.

---

© 2026 Andrés Antonio Cardoso — GLASTOR-DEV / GLASTOR® Core Organization.
