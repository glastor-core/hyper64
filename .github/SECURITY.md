# 🛡️ Política de Seguridad (Security Policy) — GLASTOR® Strata

En el proyecto **GLASTOR® Strata** (`glastor-core/strata`), desarrollado por `GLASTOR-DEV` (división de ingeniería de **GLASTOR® Core**), la seguridad del software, la protección de la cadena de suministro y la integridad del ecosistema Rust son prioridades fundamentales.

Todos los crates de la workspace se compilan con directiva estricta `#![forbid(unsafe_code)]`.

---

## 📦 Versiones Soportadas

Mantenemos actualizaciones de seguridad activas para las versiones estables de Strata:

| Versión | Estado de Soporte | Tipo de Soporte |
| :--- | :--- | :--- |
| `0.1.x` | :white_check_mark: | Soporte activo y parches inmediatos |
| `< 0.1` | :x: No soportada | Versiones de desarrollo previas |

---

## 🚨 Reporte Responsable de Vulnerabilidades

Si descubres una vulnerabilidad de seguridad o un fallo que comprometa la integridad de Strata (por ejemplo: evasión del sanitizador de valores arbitrarios de CSS, inyección de scripts, vulnerabilidad de salto de directorio *Path Traversal* en el inyector de componentes CLI o desbordamiento en FSMs), te solicitamos **no abrir un issue público**. En su lugar, activa nuestro protocolo de divulgación coordinada:

### Opción 1: GitHub Private Vulnerability Reporting (Recomendado)

Accede directamente a la herramienta oficial de GitHub: 👉
[Reportar vulnerabilidad privada en Strata](https://github.com/glastor-core/strata/security/advisories/new)

### Opción 2: Correo Electrónico Seguro

Envía los detalles técnicos a nuestro equipo de seguridad: 📧
- **[engineering@glastor.io](mailto:engineering@glastor.io)**
- **[glastor.info@gmail.com](mailto:glastor.info@gmail.com)**

### Información a Incluir en el Reporte

Para acelerar el triaje y la mitigación, incluye:
- Descripción clara del vector de ataque o bypass de seguridad.
- Pasos mínimos de reproducción o repositorio de prueba de concepto (PoC).
- Impacto potencial en tiempo de compilación (`build.rs`) o en aplicaciones WASM desplegadas.
- Parche o mitigación sugerida (si está disponible).

---

## ⏱️ SLA y Compromiso de Respuesta

- **Recepción y acuse de recibo inicial:** $\le$ 24 horas hábiles.
- **Triaje y evaluación de severidad (CVSS):** $\le$ 48 horas hábiles.
- **Publicación de parche de seguridad:** Prioridad inmediata con aviso público a través de GitHub Security Advisory y crates.io.

---

© 2026 Andrés Antonio Cardoso — GLASTOR-DEV / GLASTOR® Core Organization.
