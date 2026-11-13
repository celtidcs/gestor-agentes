# Consulta: verificación, mutaciones y registros

Documento de consulta de `AGENTS.md`. Lo escribe Gestor Agentes en cada lanzamiento; no lo edites. Léelo entero antes de verificar, aplicar una mutación de control o entregar algo para verificar.

## 1. Las cuatro puertas

En su forma equivalente para el lenguaje del proyecto:
1. La suite de pruebas completa, en verde.
2. El linter al máximo nivel de exigencia razonable, sin un solo aviso.
3. El formateador, sin una sola diferencia.
4. La auditoría de dependencias, sin vulnerabilidades conocidas ni excepciones.

- Se pasan en todas las plataformas del proyecto (Windows y Linux, salvo que sea de una sola) antes de fijar cualquier candidato de entrega.
- Las ejecuta un agente o persona distinta de quien escribió el cambio. Si solo interviene un agente, verifica otro convocado para ello o el Director.

## 2. Ejecutar, romper y comparar

- Antes de afirmar que algo está roto o arreglado, ejecútalo. Leer código y opinar no es un hallazgo.
- Después rompe a propósito lo que vigila la prueba y comprueba que se pone en rojo.
- En una reescritura o refactorización, compara la salida de antes y la de después. Fija antes lo no determinista: identificadores aleatorios, marcas de tiempo, orden de tablas hash.
- Las listas de reglas son criterios de aceptación: se aplican ejecutando, no leyendo.

## 3. Barreras separadas

Las pruebas barrera viven en un archivo distinto del código que vigilan, para que sobrevivan si la implementación se daña o se revierte.

## 4. Mutaciones de control

1. Antes de aplicar una mutación, copia el archivo en `.gestor/respaldos/`.
2. Anótala en el apartado de mutaciones vivas de `estado.md` mientras siga aplicada.
3. Restáurala solo desde esa copia o con un parche inverso exacto. Con cambios sin confirmar de varios autores, `git checkout` está prohibido: destruye el trabajo ajeno.
4. Bórrala de mutaciones vivas al restaurar.

Nadie corrige un rojo ajeno sin mirar antes las mutaciones vivas y la bandeja del dueño del archivo.

## 5. Calibrar las herramientas de medida

- Antes de creer un grep, un diff o cualquier medición, contrástalo con un caso de resultado conocido.
- Si la salida sorprende, la primera hipótesis es que la medición responde a otra pregunta.
- Las marcas se reconocen como líneas completas, nunca como subcadenas.

## 6. Registros que no se pueden inventar

- Toda verificación deja un registro en la carpeta del agente que verifica, con su suma sha256 publicada en el canal.
- El registro empieza con: nombre de la máquina, sistema (`uname` o la versión de Windows), fecha y hora, y lo que se verifica:
  - el commit, si ya existe, verificado sobre el objeto del commit (`git archive`), no sobre el árbol de trabajo; o
  - en una entrega congelada antes del commit, el commit base y la suma de la lista de la entrega, comprobada en la máquina con `sha256sum -c` o `Get-FileHash`.
- Una cifra que no cuadra con el registro invalida la verificación entera.
- Cinco de cinco no es el cien por cien: di sobre cuántos casos has medido.

## 7. Al rechazar

Si rechazas por una trampa reproducible del proyecto o de su entorno, deja la regla preventiva como dice `.gestor/consulta/metodo.md`, apartado 3.
