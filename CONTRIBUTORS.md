# 🏛️ Gobernanza, Escalabilidad y Sostenibilidad Técnica — GLASTOR® Strata

![Contributors](https://img.shields.io/github/contributors/glastor-core/strata)
![Stars](https://img.shields.io/github/stars/glastor-core/strata)
![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)
![Rust 2024](https://img.shields.io/badge/Rust-2024%20Edition-orange.svg)

`GLASTOR-DEV` (división de ingeniería de **GLASTOR® Core**) concibe **Strata** como la infraestructura definitiva de interfaz y sistema de diseño para el ecosistema **Rust WebAssembly** (Leptos, Dioxus, Yew y WASM nativo).

Nuestra arquitectura se sustenta en tres pilares inquebrantables:
1. **Zero-Node & Zero-NPM**: Eliminación total de dependencias externas de JavaScript en la compilación y CI/CD.
2. **Determinismo y Accesibilidad Estricta (WCAG 2.2 / WAI-ARIA 1.2)**: FSMs puras que previenen violaciones de accesibilidad a nivel de sistema de tipos.
3. **Rendimiento Vectorial y Memoria Compacta**: Estados FSM de $\le 1$ byte y transiciones sub-microsegundo certificadas por `byte-gauge`.

---

### 👥 Liderazgo y Dirección de Ingeniería

El desarrollo y dirección estratégica de GLASTOR® Strata es coordinado por:

- **Andrés Antonio Cardoso** ([@glastor-dev](https://github.com/glastor-dev))
  - *Lead Software Architect & Project Maintainer*
  - Responsable de la visión arquitectónica, la gobernanza de crates y la interoperabilidad con `byte-gauge`, `specter` y `zerho`.

---

### 🚀 Framework de Contribución (Rust-First DX)

Incentivamos una **Developer Experience (DX)** de clase mundial para la comunidad de código abierto:

1. **Compilación en Rust Puro**: Todo se construye mediante `cargo build`, `cargo test` y `cargo clippy`.
2. **Seguridad Absoluta**: `#![forbid(unsafe_code)]` activo en todos los crates de la workspace.
3. **Contratos Auditados con Specter**: Ningún cambio puede romper interfaces de componentes sin pasar la verificación de AST diffing.
4. **Protección de Rama Principal (`main`)**: Todos los aportes deben pasar por Pull Request con revisión y suite de CI verde (formato, clippy, 30 tests unitarios/integrales).

Para contribuir con nuevas primitivas FSM, optimizaciones en el compilador de CSS o integraciones con frameworks, consulta nuestra guía en [.github/CONTRIBUTING.md](.github/CONTRIBUTING.md) y abre un Pull Request hacia la rama `main`.

---

### 💎 Ecosistema GLASTOR® Core

| Módulo / Crate | Propósito | Integración |
| :--- | :--- | :--- |
| **`glastor-aria-core`** | FSMs headless WAI-ARIA 1.2 (`Dialog`, `Tabs`, `Toast`, `Grid`, `Accordion`, `Popover`) | Validación de memoria con [`byte-gauge`](https://github.com/glastor-core/byte-gauge) |
| **`glastor-css-binder`** | Escáner de clases utilitarias, motor de variantes y física de resortes (`glastor-motion`) | Zero-Node en `build.rs` y `watch` daemon |
| **`glastor-ui-cli`** | Distribuidor de componentes Cargo (`cargo ui` / `cargo strata`) y sincronizador DTCG | Contratos verificados con [`specter`](https://github.com/glastor-core/specter) y docs en [`zerho`](https://github.com/glastor-core/zerho) |

---

© 2026 Andrés Antonio Cardoso — GLASTOR-DEV / GLASTOR® Core Organization.
