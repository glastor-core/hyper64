# Changelog — GLASTOR® hyper64

<p align="center">
  <img src="https://img.shields.io/badge/Format-Keep%20a%20Changelog-blue.svg" alt="Keep a Changelog">
  <img src="https://img.shields.io/badge/Versioning-SemVer%202.0.0-green.svg" alt="Semantic Versioning">
  <img src="https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg" alt="License">
  <img src="https://img.shields.io/badge/Repository-glastor--core%2Fhyper64-informational" alt="Repository">
</p>

---

## Acerca de este Changelog

Todos los cambios notables en **GLASTOR® hyper64** (Next-Generation, Ultra-High-Throughput, Zero-Allocation Base64 Engine for Rust) son documentados en este archivo.

Este proyecto se adhiere a:
- **Formato:** [Keep a Changelog](https://keepachangelog.com/es-ES/1.0.0/)
- **Versionado:** [Semantic Versioning](https://semver.org/spec/v2.0.0.html)

---

## [0.1.0] - 2026-11-11

### ✨ Added

#### Motor SIMD Vectorizado Multi-Nivel
- **Aceleración AVX2 (256-bit)**: Procesamiento de 32 bytes Base64 por ciclo con permutación aritmética vectorizada para codificación y decodificación, alcanzando rendimientos empíricos superiores a **10.20 GiB/s** (+340% sobre `base64` 0.22).
- **Fallback SSSE3 (128-bit)**: Vectorización para procesadores x86_64 anteriores o sin soporte AVX2 (16 bytes/ciclo).
- **Aceleración ARM64 NEON**: Soporte nativo para arquitecturas AArch64 (Apple Silicon, AWS Graviton, Raspberry Pi 4/5).
- **Detección Dinámica de CPU en `#![no_std]`**: Detección de AVX2/SSSE3 sin dependencia de `std` mediante `core::arch::x86_64::__cpuid` y validación de contexto de registros `_xgetbv(0)` para SOs bare-metal y enclaves SGX.

#### Ergonomía y Memoria de Cero Asignaciones
- **Decodificación In-Place (`decode_in_place`)**: Decodificación directa sobre el búfer mutable del llamador con **0 bytes asignados en el heap**, con demostración formal matemática de seguridad ($r_k \ge w_k$).
- **Eliminación de Doble Escritura**: Asignación directa sobre la capacidad reservada (`Vec::with_capacity` y `from_raw_parts_mut`) para `encode()` y `decode()`, reduciendo a la mitad la saturación del bus de memoria en payloads masivos.
- **Formateador `Display` de Cero Asignaciones (`hyper64::display`)**: Formateo directo sobre flujos `core::fmt` usando un búfer de 64 bytes en stack (1 línea de caché L1).
- **Streaming de Chunks (`Base64ChunkIter` / `encode_iter`)**: Iterador de chunks fijos `[u8; 4]` para transmisión por paquetes sin heap.

#### Seguridad y Criptografía
- **Modo en Tiempo Constante (`encode_constant_time`, `decode_constant_time`, `ct_eq`)**: Eliminación completa de tablas de consulta (LUTs) y saltos condicionales dependientes de datos secretos para mitigar ataques de canal lateral (*cache-timing attacks*), reforzado con barreras de optimización `core::hint::black_box`.
- **Parser PEM / MIME con Fast-Path SWAR**: Decodificación tolerante a espacios y saltos de línea (`\r\n`, `\n`, espacios, tabs) directamente in-place con escáner SWAR de 64 bits para bloques contiguos de certificados X.509 y claves SSH.

#### Integraciones de Ecosistema
- **Compatibilidad 100% Drop-in con `base64` 0.22**: Capa completa con `Engine`, `GeneralPurpose`, `prelude::*` y `general_purpose::STANDARD`.
- **I/O Síncrono y Asíncrono**: `EncoderWriter` y `DecoderReader` para `std::io`, y `AsyncEncoderWriter` y `AsyncDecoderReader` para `tokio::io::AsyncWrite` / `AsyncRead`.
- **Integración Serde**: Módulos `#[serde(with = "...")]` y envoltorio `Base64<T>` utilizando `serializer.collect_str` sin strings intermedios.
- **Evaluación en Tiempo de Compilación (`const fn`)**: Funciones `encode_to_array`, `decode_to_array` y macros `encode_const!`, `decode_const!`.

#### Garantías y Auditoría CI/CD
- **Demostración Formal de Seguridad (`SAFETY.md`)**: Demostración matemática del avance de punteros y justificación de todos los bloques `unsafe`.
- **Infraestructura de Auditoría**: Workflows de CI multi-OS, verificación de Undefined Behavior con Miri, fuzzing continuo con libFuzzer + ASan, matriz multi-arquitectura (`cross`), `cargo-deny`, y acciones `glastor-core` (`specter`, `byte-gauge`, `vortex`).

---

© 2026 Andrés Antonio Cardoso — GLASTOR-DEV / GLASTOR® Core Organization.
