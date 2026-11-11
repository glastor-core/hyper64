## 📋 Descripción del Cambio

<!-- Explica de forma concisa qué problema resuelve este Pull Request o qué funcionalidad agrega a GLASTOR® Strata. -->

## 🎯 Tipo de Cambio

- [ ] 🐛 Bug fix (corrección de error sin breaking changes)
- [ ] ✨ Nueva primitiva FSM / Característica (nueva funcionalidad compatible hacia atrás)
- [ ] 💥 Breaking change (cambio que altera interfaces públicas o FSMs previas)
- [ ] ⚡ Optimización de rendimiento (reducción de huella de memoria o micro-benchmarks)
- [ ] 📝 Documentación (actualización de especificaciones, `llms.txt` o README)
- [ ] 🔧 Mantenimiento / CI / Tooling (refactorización, Clippy, GitHub Actions)

## 🔗 Issues Relacionados

<!-- Vincula los issues afectados, por ejemplo: Fixes #123, Closes #456 -->

- Fixes #

## 🧪 Pruebas Realizadas

<!-- Marca las verificaciones ejecutadas localmente antes de enviar el PR: -->

- [ ] Formateo estricto: `cargo fmt --all --check`
- [ ] Linter Clippy sin advertencias: `cargo clippy --workspace --all-targets --all-features`
- [ ] Suite de pruebas completa: `cargo test --workspace --all-features`
- [ ] Auditoría de memoria con byte-gauge: `cargo test -p glastor-aria-core --test memory_footprint`
- [ ] Verificación de contratos con specter: `cargo run --bin cargo-ui -- audit`
- [ ] Generación de documentación: `cargo doc --workspace --no-deps`

## 🛡️ Checklist de Protección de Rama (`main`)

- [ ] Mi rama parte de la versión más reciente de `main`.
- [ ] Se mantiene el estándar `#![forbid(unsafe_code)]` en todos los archivos modificados.
- [ ] Los mensajes de commit cumplen con [COMMIT_CONVENTION.md](./COMMIT_CONVENTION.md).
- [ ] He actualizado el archivo `CHANGELOG.md` si corresponde.
- [ ] No se introducen dependencias de Node.js, npm ni código no auditado.

---

© 2026 Andrés Antonio Cardoso — GLASTOR-DEV / GLASTOR® Core Organization.
