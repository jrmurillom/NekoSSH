# Mi Primer Tarea (Guía Rápida para Devs)

Esta es tu hoja de trucos. Aquí está el flujo exacto, paso a paso, de cómo tomar una tarea desde que el PM la definió hasta que la entregas terminada. Sin teoría, solo ejecución.

Ciclo completo y alias de comandos: [`docs/workflow-ciclo-end-to-end.md`](../workflow-ciclo-end-to-end.md).

---

### 1. Cargar Contexto
El PM ya preparó la tarea en ZenTao. Tú solo descargas el contexto a tu entorno local:
```bash
init-task <task-id>
```
* **Iteración:** ❌ No. Se corre **una sola vez** por tarea.
* **Nota:** Copia todo el texto que te escupe la terminal para dárselo a Cursor en el siguiente paso.

---

### 2. Proponer Solución Técnica
Ve al chat de Cursor (o Claude) y ejecuta:
```text
/opsx:propose task-<id>-<nombre-corto>
```
* **Iteración:** 🔄 **Sí.** 
* **Nota:** Revisa el diseño técnico (`design.md`) y la lista de tareas (`tasks.md`) que genera el agente. Si no te convence la arquitectura o falta algo, pídele al agente ahí mismo en el chat que lo ajuste. **No pases al siguiente paso hasta que el plan técnico sea perfecto.**

---

### 3. Implementación Continua (El código)
Cuando el plan esté aprobado, ejecuta:
```text
/opsx:apply task-<id>-<nombre-corto>
```
* **Iteración:** 🔄 **Sí.** 
* **Nota:** El agente creará automáticamente la rama de Git (`feature/...`) y empezará a programar tarea por tarea siguiendo las reglas de TDD (Test-Driven Development). Deja que trabaje.

---

### 🚧 Parada de Emergencia (Solo si algo sale mal)
¿Chocaste con un muro técnico, una librería está rota, o te diste cuenta de que el diseño inicial era malo? Pausa al agente y en el chat ejecuta:
```text
/opsx:fix "Describe por qué falló el plan y cómo quieres solucionarlo"
```
* **Iteración:** 🔄 **Sí.**
* **Nota:** El agente recalculará la ruta técnica (actualizará el `design.md` y `tasks.md` en caliente) **sin borrar tu progreso previo**. Cuando termine de parchar el plan, simplemente vuelve a lanzar `/opsx:apply` y continuará construyendo.

---

### 4. Guardar Cambios (Commit y PR)
Una vez que el código funciona y pasaste las pruebas manuales, guarda tu trabajo:
```text
commit task-<id>
```
* **Iteración:** ❌ No.
* **Nota:** Es un skill automatizado. Evaluará todo lo que hiciste, generará el mensaje de commit semántico perfecto, hará push a la rama y abrirá el Pull Request de forma automática.

---

### 5. Archivar y Limpiar
```text
/opsx:archive task-<id>-<nombre-corto>
```
* **Iteración:** ❌ No.
* **Nota:** El agente consolidará todo el conocimiento nuevo en la documentación global (`docs/`) para que el equipo esté sincronizado, y cerrará tu ciclo de trabajo.

¡Listo! Ya puedes tomar la siguiente tarea.
