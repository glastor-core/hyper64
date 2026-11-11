# 🛡️ Política de Seguridad (Security Policy) — GLASTOR® hyper64

En el proyecto **GLASTOR® hyper64** (`glastor-core/hyper64`), desarrollado por `GLASTOR-DEV` (división de ingeniería de **GLASTOR® Core**), la seguridad de la memoria, la inmunidad a canales laterales criptográficos y la integridad de la cadena de suministro son prioridades inquebrantables.

Todo el código que utiliza bloques `unsafe` para vectorización SIMD o decodificación *in-place* está sujeto a demostraciones matemáticas estrictas documentadas en [SAFETY.md](../SAFETY.md), verificado continuamente con **Miri** y auditado con **AddressSanitizer (ASan)** mediante **libFuzzer**.

---

## 📦 Versiones Soportadas

Mantenemos actualizaciones de seguridad activas para las versiones estables de hyper64:

| Versión | Estado de Soporte | Tipo de Soporte |
| :--- | :--- | :--- |
| `0.1.x` | :white_check_mark: | Soporte activo y parches inmediatos |
| `< 0.1` | :x: No soportada | Versiones de desarrollo previas |

---

## 🚨 Reporte Responsable de Vulnerabilidades

Si descubres una vulnerabilidad de seguridad o un fallo que comprometa la integridad de hyper64 (por ejemplo: desbordamiento de búfer en decodificación in-place, violación del invariante $r_k \ge w_k$, fuga de información por canal lateral en las rutinas de tiempo constante, o degradación de seguridad en la inicialización de memoria), te solicitamos **no abrir un issue público**. En su lugar, activa nuestro protocolo de divulgación coordinada:

### Opción 1: GitHub Private Vulnerability Reporting (Recomendado)

Accede directamente a la herramienta oficial de GitHub:
👉 [Reportar vulnerabilidad privada en hyper64](https://github.com/glastor-core/hyper64/security/advisories/new)

### Opción 2: Correo Electrónico Seguro

Envía los detalles técnicos a nuestro equipo de seguridad:
📧 **[engineering@glastor.io](mailto:engineering@glastor.io)** o **[glastor.info@gmail.com](mailto:glastor.info@gmail.com)**

### Información a Incluir en el Reporte

Para acelerar el triaje y la mitigación, incluye:
- Descripción clara del vector de ataque o vulnerabilidad de memoria.
- Pasos mínimos de reproducción o prueba de concepto (PoC) en Rust.
- Arquitectura afectada (`x86_64`, `aarch64`, `s390x`, `no_std`).
- Parche o mitigación sugerida (si está disponible).

---

## ⏱️ SLA y Compromiso de Respuesta

- **Recepción y acuse de recibo inicial:** $\le$ 24 horas hábiles.
- **Triaje y evaluación de severidad (CVSS):** $\le$ 48 horas hábiles.
- **Publicación de parche de seguridad:** Prioridad inmediata con aviso público a través de GitHub Security Advisory y crates.io.

---

© 2026 Andrés Antonio Cardoso — GLASTOR-DEV / GLASTOR® Core Organization.
