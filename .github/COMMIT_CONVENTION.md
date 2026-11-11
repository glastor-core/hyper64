# 🏛️ Convención de Mensajes de Commit (Conventional Commits) — GLASTOR® hyper64

Para escalar el desarrollo y garantizar una **Software Supply Chain** transparente, este repositorio implementa una convención estricta basada en el estándar [Conventional Commits 1.0.0](https://www.conventionalcommits.org/).

Esta práctica impulsa la automatización de versiones con SemVer, la generación de changelogs y la auditoría de contratos con `specter`.

---

## ⚙️ Estructura del Mensaje

```text
<tipo>(<scope>): <asunto en minúsculas y modo imperativo>

[cuerpo descriptivo opcional]

[pie de página / referencias opcional]
```

### 1. Tipos de Commit Permitidos

| Tipo | Propósito | Impacto en Release |
| :--- | :--- | :---: |
| `feat` | Nueva funcionalidad, extensión SIMD o soporte de API | Minor (`v0.X.0`) |
| `fix` | Corrección de bug o discrepancia en decodificación/padding | Patch (`v0.0.X`) |
| `perf` | Optimización de ciclos SIMD, vectorización o buffers | Patch (`v0.0.X`) |
| `refactor` | Reestructuración interna sin alterar comportamiento público | Ninguno |
| `docs` | Actualización de documentación, SAFETY.md o README | Ninguno |
| `test` | Nuevas pruebas unitarias, oráculos o fuzz targets | Ninguno |
| `ci` | Ajustes en GitHub Actions, workflows o automatizaciones | Ninguno |
| `chore` | Tareas de mantenimiento, dependencias o metadatos | Ninguno |

### 2. Scopes Oficiales de hyper64

- `simd`: Vectorización AVX2, SSSE3, AArch64 NEON o AVX-512 (`src/simd.rs`).
- `decode`: Decodificación general, in-place y de slices (`src/decode.rs`).
- `encode`: Codificación general, streaming de chunks y buffers (`src/encode.rs`).
- `permissive`: Parser PEM, MIME y tolerancia a whitespace con SWAR (`src/permissive.rs`).
- `crypto`: Modo en tiempo constante y mitigación de canal lateral (`src/constant_time.rs`).
- `engine`: Capa de compatibilidad drop-in con `base64::Engine` (`src/engine.rs`).
- `io`: Adaptadores de streaming síncrono `std::io` y asíncrono Tokio (`src/io.rs`).
- `serde`: Módulos de serialización y deserialización zero-copy (`src/serde.rs`).
- `const`: Funciones y macros en tiempo de compilación (`src/const_fn.rs`).
- `fuzz`: Objetivos y arneses de libFuzzer (`fuzz/`).

---

## 💡 Ejemplos Válidos

```text
feat(simd): add AVX-512BW vectorized decoding fast-path
fix(permissive): handle trailing whitespace preceding certificate footer
perf(encode): eliminate zero-filling overhead via spare capacity buffer
docs(safety): formally prove the r >= w in-place decoding invariant
ci(miri): enforce stacked borrows checks in continuous integration
```

---

© 2026 Andrés Antonio Cardoso — GLASTOR-DEV / GLASTOR® Core Organization.
