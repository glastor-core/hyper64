# 🌌 Guía de Contribución y Estándares de Ingeniería — GLASTOR® hyper64

Gracias por sumarte a la misión de construir y optimizar **GLASTOR® hyper64** dentro del ecosistema de `GLASTOR-DEV` (división de ingeniería de **GLASTOR® Core**).

`hyper64` es el motor de Base64 definitivo de ultra alto rendimiento, cero asignaciones de memoria y tiempo constante para Rust (sistemas distribuidos, microservicios, almacenamiento masivo y criptografía), eliminando por completo las deficiencias del crate estándar.

---

## 🛡️ Política de Protección de Rama (`main`)

Para garantizar la máxima estabilidad y la calidad del código en producción:
- **La rama `main` está protegida**: No se permiten envíos directos (`git push origin main`) ni modificaciones forzadas (`--force`).
- **Todo cambio debe ingresar mediante Pull Request (PR)**:
  - Debe crearse una rama temática (`feat/nombre`, `fix/nombre`, `perf/nombre`).
  - La suite automatizada de GitHub Actions (tests unitarios/integración, matriz multi-OS, Clippy, doctests, Miri, Fuzzing y auditoría Specter) debe finalizar en estado verde (`[PASS]`).
  - Se requiere al menos una revisión aprobada por los mantenedores ([CODEOWNERS](./CODEOWNERS)).

---

## 💻 Setup del Entorno de Desarrollo

### Requisitos Previos

- **Rust**: Versión estable 1.80+ (Edición 2024 soportada).
- **Git**: 2.30+.
- **Zero Dependencies**: Cero dependencias externas requeridas en compilación por defecto.

### 1. Clonación del Repositorio

```bash
git clone https://github.com/glastor-core/hyper64.git
cd hyper64
```

### 2. Comandos de Trabajo y Verificación

```bash
# Compilar la biblioteca y tests
cargo build --all-targets --all-features

# Ejecutar todas las pruebas unitarias y de integración
cargo test --all-features

# Validar soporte puro en #![no_std] (sin alloc ni std)
cargo test --lib --no-default-features

# Validar que los ejemplos en README y doctests compilen
cargo test --doc

# Verificar cumplimiento de formato oficial
cargo fmt --all --check

# Ejecutar auditoría estricta de Clippy
cargo clippy --all-targets --all-features

# Ejecutar benchmarks con Criterion
cargo bench

# Verificar aliasing y ausencia de Undefined Behavior con Miri
cargo miri test --test in_place --test constant_time --test rfc4648

# Auditar licencias y supply chain con cargo-deny
cargo-deny check
```

---

## 📐 Estándares de Código y Manejo de `unsafe`

1. **Mandato de Seguridad Formal**:
   - Todo bloque `unsafe` agregado debe ir acompañado de un comentario `// SAFETY:` exhaustivo que justifique por qué la operación es segura.
   - Si se introducen manipulaciones de punteros en decodificación in-place, deben respetar el teorema $r_k \ge w_k$ demostrado en [SAFETY.md](../SAFETY.md).
2. **Cero Dependencias en Producción**:
   - El crate central no debe incorporar dependencias externas para funciones de codificación, decodificación o SIMD. Los features como `serde` o `tokio` deben permanecer 100% opcionales.
3. **Compatibilidad Estricta de API**:
   - Las interfaces del módulo `engine` deben preservar el 100% de compatibilidad con `base64` 0.22.

---

## 🤝 Proceso de Envío de Pull Requests

1. Crea una rama descriptiva a partir de la versión más reciente de `main`:
   ```bash
   git checkout main
   git pull origin main
   git checkout -b feat/avx512-encoding
   ```
2. Realiza tus cambios asegurando que las pruebas existentes y nuevas pasen al 100%.
3. Escribe mensajes de commit bajo el estándar [Conventional Commits](./COMMIT_CONVENTION.md).
4. Abre el Pull Request en GitHub seleccionando como destino la rama `main` y completa el formulario de [PULL_REQUEST_TEMPLATE.md](./PULL_REQUEST_TEMPLATE.md).

---

© 2026 Andrés Antonio Cardoso — GLASTOR-DEV / GLASTOR® Core Organization.
