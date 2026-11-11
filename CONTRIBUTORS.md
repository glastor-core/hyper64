# 🏛️ Gobernanza, Escalabilidad y Sostenibilidad Técnica — GLASTOR® hyper64

![Contributors](https://img.shields.io/github/contributors/glastor-core/hyper64)
![Stars](https://img.shields.io/github/stars/glastor-core/hyper64)
![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)
![Rust 2024](https://img.shields.io/badge/Rust-2024%20Edition-orange.svg)

`GLASTOR-DEV` (división de ingeniería de **GLASTOR® Core**) concibe **hyper64** como la infraestructura definitiva de codificación y decodificación Base64 de ultra alto rendimiento para el ecosistema **Rust** (sistemas distribuidos, proxies gRPC, almacenamiento masivo, criptografía y microservicios).

Nuestra arquitectura se sustenta en tres pilares inquebrantables:
1. **Rendimiento SIMD Extremo y Cero Asignación**: Vectorización multinivel (AVX2, SSSE3, NEON) que supera los 10 GiB/s y decodificación *in-place* sin tocar el heap.
2. **Seguridad Criptográfica y Formal**: Operación en tiempo constante sin LUTs para evitar ataques de canal lateral, e invariantes de punteros ($r_k \ge w_k$) matemáticamente demostrados en `SAFETY.md`.
3. **Compatibilidad y Cero Fricción**: Reemplazo *drop-in* completo de `base64` 0.22, soporte nativo de `#![no_std]` y cero dependencias de producción.

---

### 👥 Liderazgo y Dirección de Ingeniería

El desarrollo y dirección estratégica de GLASTOR® hyper64 es coordinado por:

- **Andrés Antonio Cardoso** ([@glastor-dev](https://github.com/glastor-dev))
  - *Lead Software Architect & Project Maintainer*
  - Responsable de la visión arquitectónica, la vectorización de bajo nivel y la interoperabilidad con el ecosistema `glastor-core` (`specter`, `byte-gauge`, `vortex`).

---

### 🚀 Framework de Contribución (Rust-First DX)

Incentivamos una **Developer Experience (DX)** de clase mundial para la comunidad de código abierto:

1. **Compilación en Rust Puro**: Cero dependencias externas en producción. Todo se construye y valida mediante `cargo build`, `cargo test` y `cargo clippy`.
2. **Seguridad de Memoria Verificada**: Todo bloque `unsafe` debe satisfacer estrictamente los invariantes formales de [SAFETY.md](SAFETY.md) y pasar la suite de Miri (`cargo miri test`).
3. **Contratos Auditados con Specter**: Ningún cambio puede romper firmas públicas del trait `Engine` o funciones públicas sin pasar la verificación de SemVer de `glastor-core/specter`.
4. **Protección de Rama Principal (`main`)**: Todos los aportes deben canalizarse mediante Pull Request con revisión aprobada y la suite completa de CI en verde.

Para contribuir con nuevas extensiones SIMD (como AVX-512), optimizaciones escalares o adaptadores de I/O, consulta nuestra guía en [.github/CONTRIBUTING.md](.github/CONTRIBUTING.md) y abre un Pull Request hacia la rama `main`.

---

### 💎 Ecosistema GLASTOR® Core

| Módulo / Crate | Propósito | Integración |
| :--- | :--- | :--- |
| **`hyper64`** | Motor Base64 ultra-rápido, zero-alloc y tiempo constante | Certificado por [`specter`](https://github.com/glastor-core/specter) y [`byte-gauge`](https://github.com/glastor-core/byte-gauge) |
| **`vortex`** | Orquestación predictiva de CI y guardián de regresión de rendimiento | CI en `.github/workflows/ci.yml` |
| **`specter`** | Diffing de contratos de API y cumplimiento estricto de SemVer | CI en `.github/workflows/audit.yml` |
| **`byte-gauge`** | Auditoría de hardening binario, entropía y secciones de memoria | CI en `.github/workflows/audit.yml` |

---

© 2026 Andrés Antonio Cardoso — GLASTOR-DEV / GLASTOR® Core Organization.
