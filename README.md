# Joc de Vocabulari Anglès-Català

Joc web interactiu per practicar vocabulari anglès-català. Ideal per preparar exàmens de vocabulari.

## Característiques

- **Tres modalitats**: Anglès→Català, Català→Anglès, o Barrejat
- **Configurable**: Tria el nombre de preguntes (5, 10, 15, 20 o totes)
- **Estadístiques**: Guarda encerts i errors per paraula i direcció
- **Paraules difícils**: Les paraules amb més errors apareixen més sovint
- **Tolerant amb errors**: Accepta petits errors d'escriptura i ignora accents
- **Vocabulari extern**: Fàcil de canviar la llista de paraules

## Com usar-lo

### 1. Crear o editar la llista de paraules

Edita el fitxer `vocabulari.yaml` amb qualsevol editor de text. El format és:

```yaml
titulo: El títol del vocabulari
descripcion: Una breu descripció

paraules:
  - en: cat
    ca: [gat]

  - en: dog
    ca: [gos, gossa]

  - en: house
    ca: [casa]
    altEn: [home]  # alternatives en anglès (opcional)
```

**Camps:**
- `titulo`: El títol que es mostrarà al joc
- `descripcion`: Descripció que apareix sota el títol
- `paraules`: Llista de paraules
  - `en`: Paraula en anglès (obligatori)
  - `ca`: Llista de traduccions en català acceptades (obligatori)
  - `altEn`: Llista d'alternatives en anglès (opcional)

**Consells:**
- Posa diverses traduccions si una paraula té sinònims: `ca: [gos, gossa, gossot]`
- Usa `altEn` per acceptar diferents formes: `altEn: [pajamas]` per a `pyjamas`
- Els accents i majúscules s'ignoren automàticament

### 2. Llençar el servidor web

El joc necessita un servidor web per carregar el fitxer YAML. Aquí tens les instruccions per a cada sistema operatiu:

---

#### Linux

```bash
cd /ruta/al/joc-vocabulari-angles
sudo python3 -m http.server 80
```

O sense permisos de root (port 8000):
```bash
python3 -m http.server 8000
```

Obre el navegador a: `http://localhost` (o `http://localhost:8000`)

---

#### macOS

```bash
cd /ruta/al/joc-vocabulari-angles
python3 -m http.server 8000
```

O amb permisos d'administrador per usar el port 80:
```bash
sudo python3 -m http.server 80
```

Obre el navegador a: `http://localhost:8000` (o `http://localhost`)

---

#### Windows + WSL (recomanat)

1. Obre una terminal WSL (Ubuntu, Debian, etc.)
2. Navega a la carpeta del joc:
```bash
cd /mnt/c/Users/NomUsuari/Documents/joc-vocabulari-angles
```
3. Llença el servidor:
```bash
python3 -m http.server 8000
```

Obre el navegador a: `http://localhost:8000`

---

#### Windows (sense WSL)

**Opció 1: Python**

1. Instal·la Python des de https://www.python.org/downloads/
2. Obre el Command Prompt (cmd) o PowerShell
3. Navega a la carpeta del joc:
```cmd
cd C:\Users\NomUsuari\Documents\joc-vocabulari-angles
```
4. Llença el servidor:
```cmd
python -m http.server 8000
```

Obre el navegador a: `http://localhost:8000`

**Opció 2: Node.js**

1. Instal·la Node.js des de https://nodejs.org/
2. Instal·la el servidor:
```cmd
npm install -g http-server
```
3. Navega a la carpeta i llença:
```cmd
cd C:\Users\NomUsuari\Documents\joc-vocabulari-angles
http-server -p 8000
```

---

### 3. Accedir des d'altres dispositius (mòbil, tablet)

Si vols que el nen practiqui des d'un altre dispositiu a la mateixa xarxa:

1. Troba la IP del teu ordinador:
   - **Linux/Mac**: `ip addr` o `ifconfig`
   - **Windows**: `ipconfig`

2. Llença el servidor al port 80:
```bash
sudo python3 -m http.server 80
```

3. Des del mòbil/tablet, obre: `http://IP_DEL_ORDINADOR`
   - Exemple: `http://192.168.1.50`

---

## Estructura de fitxers

```
joc-vocabulari-angles/
├── index.html        # El joc (no cal tocar-lo)
├── vocabulari.yaml   # Llista de paraules (editable)
└── README.md         # Aquest fitxer
```

## Crear múltiples llistes de vocabulari

Pots tenir diverses llistes de vocabulari i canviar entre elles:

1. Crea fitxers com `animals.yaml`, `colors.yaml`, `verbs.yaml`
2. Quan vulguis canviar de llista, renombra o copia:
```bash
cp animals.yaml vocabulari.yaml
```
3. Refresca el navegador

## Esborrar estadístiques

Les estadístiques es guarden al navegador (localStorage). Per esborrar-les:
- Clica "Esborrar historial" a la pantalla d'inici, o
- Obre les eines de desenvolupador del navegador (F12) → Application → Local Storage → Eliminar

## Llicència

MIT License - Lliure per usar, modificar i distribuir.
