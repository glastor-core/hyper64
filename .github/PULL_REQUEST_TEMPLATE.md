## 📋 Descripción del Cambio

<!-- Explica de forma concisa qué problema resuelve este Pull Request o qué funcionalidad optimiza en GLASTOR® hyper64. -->

## 🎯 Tipo de Cambio

- [ ] 🐛 Bug fix (corrección de error sin breaking changes en la decodificación o padding)
- [ ] ✨ Nueva característica / extensión SIMD (ej: AVX-512, nuevos alfabetos o adaptadores)
- [ ] 💥 Breaking change (cambio que altera interfaces públicas del Engine o firmas de funciones)
- [ ] ⚡ Optimización de rendimiento (mejora de micro-benchmarks o reducción de ciclos SIMD)
- [ ] 📝 Documentación (actualización de README, SAFETY.md o doctests)
- [ ] 🔧 Mantenimiento / CI / Tooling (refactorización, Clippy, workflows de GitHub Actions)

## 🔗 Issues Relacionados

<!-- Vincula los issues afectados, por ejemplo: Fixes #123, Closes #456 -->

- Fixes #

## 🧪 Pruebas Realizadas

<!-- Marca las verificaciones ejecutadas localmente antes de enviar el PR: -->

- [ ] Formateo estricto: `cargo fmt --all --check`
- [ ] Linter Clippy sin advertencias: `cargo clippy --all-targets --all-features`
- [ ] Suite de pruebas completa: `cargo test --all-features`
- [ ] Verificación en `no_std`: `cargo test --lib --no-default-features`
- [ ] Validación de doctests: `cargo test --doc`
- [ ] Verificación de aliasing y UB con Miri: `cargo miri test --test in_place --test constant_time --test rfc4648`
- [ ] Auditoría de licencias y dependencias: `cargo-deny check`

## 🛡️ Checklist de Protección de Rama (`main`)

- [ ] Mi rama parte de la versión más reciente de `main`.
- [ ] Todo bloque `unsafe` agregado cuenta con una justificación de seguridad y respeta los invariantes de `SAFETY.md`.
- [ ] Los mensajes de commit cumplen con [COMMIT_CONVENTION.md](./COMMIT_CONVENTION.md).
- [ ] He actualizado el archivo `CHANGELOG.md` si corresponde.
- [ ] No se introducen dependencias de producción externas innecesarias.

---

© 2026 Andrés Antonio Cardoso — GLASTOR-DEV / GLASTOR® Core Organization.
