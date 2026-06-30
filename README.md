# Task Manager — Gestor del Día a Día

Monorepo con un backend REST en Rust y tres clientes (web, móvil y escritorio) para gestionar tareas personales con subtareas, metadatos y autenticación por usuario.

## Arquitectura

```
task-manager/
├── core-backend/      # API REST (Axum + PostgreSQL)
├── client-web/        # Cliente React + Vite
├── client-mobile/     # Cliente React Native (Android / iOS)
└── client-desktop/    # Cliente GTK 4 + Libadwaita (Linux)
    └── mate-applet/   # Applet MATE opcional para abrir el escritorio
```

Todos los clientes consumen la misma API en `http://localhost:5040` (configurable).

## Características

- **Autenticación JWT** con access token y refresh token rotativo.
- **Usuarios** con roles: administrador y usuario normal.
- **Tareas** por usuario; el admin puede ver todas las tareas del sistema.
- **Subtareas** mediante `parent_id`, expuestas en la API como árbol con `children`.
- **Edición** de título, descripción, estado, metadatos y padre.
- **Metadatos** opcionales: prioridad (`baja` | `media` | `alta`), fecha límite y etiquetas.
- **Administración de usuarios** (solo admin) desde web, móvil y escritorio.
- **Tema claro/oscuro** en web y móvil.

## Requisitos

| Componente      | Herramientas                                      |
|-----------------|---------------------------------------------------|
| Backend         | Rust (edition 2021), PostgreSQL 14+             |
| Cliente web     | Node.js 18+, npm                                  |
| Cliente móvil   | Node.js 22+, React Native CLI, Android SDK / Xcode |
| Cliente desktop | Rust, GTK 4, Libadwaita (Linux)                   |

## Base de datos

1. Crea la base de datos:

```bash
createdb tasks_db
```

2. Aplica el esquema inicial:

```bash
psql -d tasks_db -f core-backend/init.sql
```

Si migras una base antigua sin usuarios, revisa también `core-backend/migrations/`.

Al arrancar el backend, el usuario `admin` recibe la contraseña definida en `ADMIN_INITIAL_PASSWORD` (por defecto `changeme` si no se configura).

## Backend (`core-backend`)

```bash
cd core-backend
cp .env.example .env   # ajusta DB_* y JWT_SECRET
cargo run
```

El servidor escucha en **http://0.0.0.0:5040**.

### Variables de entorno

| Variable                 | Descripción                          | Default        |
|--------------------------|--------------------------------------|----------------|
| `DB_HOST`                | Host de PostgreSQL                   | `localhost`    |
| `DB_PORT`                | Puerto                               | `5432`         |
| `DB_USER`                | Usuario                              | `postgres`     |
| `DB_PASSWORD`            | Contraseña                           | `postgre`      |
| `DB_NAME`                | Nombre de la base                    | `tasks_db`     |
| `JWT_SECRET`             | Secreto para firmar tokens           | *(dev only)*   |
| `JWT_ACCESS_TTL_SECS`    | Caducidad del access token (s)       | `3600`         |
| `JWT_REFRESH_TTL_SECS`   | Caducidad del refresh token (s)      | `604800`       |
| `ADMIN_INITIAL_PASSWORD` | Contraseña inicial del usuario admin | `changeme`     |

## Cliente web (`client-web`)

```bash
cd client-web
npm install
npm run dev
```

Abre la URL que muestre Vite (normalmente http://localhost:5173).

Opcional: define `VITE_API_BASE_URL` si el backend no está en `http://localhost:5040`.

## Cliente móvil (`client-mobile`)

```bash
cd client-mobile
npm install
npm start          # Metro bundler
npm run android    # o npm run ios
```

En emulador Android la API apunta a `http://10.0.2.2:5040`; en iOS/simulador usa `localhost`.

## Cliente de escritorio (`client-desktop`)

```bash
cd client-desktop
cp .env.example .env   # opcional: API_BASE_URL
cargo run
```

Incluye integración D-Bus para mostrar/ocultar la ventana y un applet MATE en `client-desktop/mate-applet/`:

```bash
cd client-desktop/mate-applet
cargo run
```

## API REST

### Públicas (sin token)

| Método | Ruta             | Descripción        |
|--------|------------------|--------------------|
| POST   | `/auth/login`    | Iniciar sesión     |
| POST   | `/auth/refresh`  | Renovar tokens     |
| POST   | `/auth/logout`   | Revocar refresh    |

### Protegidas (`Authorization: Bearer <access_token>`)

| Método | Ruta                        | Descripción                          |
|--------|-----------------------------|--------------------------------------|
| GET    | `/tasks`                    | Lista de tareas raíz con `children`  |
| POST   | `/tasks`                    | Crear tarea o subtarea               |
| PUT    | `/tasks/:id`                | Actualizar tarea completa            |
| PATCH  | `/tasks/:id/description`    | Actualizar solo descripción          |
| PATCH  | `/tasks/:id/metadata`       | Actualizar solo metadatos            |
| POST   | `/tasks/:id/toggle`         | Alternar `completed`                 |
| GET    | `/users`                    | Listar usuarios (admin)              |
| POST   | `/users`                    | Crear usuario (admin)                |
| GET    | `/users/:id`                | Ver usuario (propio o admin)         |
| PUT    | `/users/:id`                | Actualizar usuario                   |
| DELETE | `/users/:id`                | Eliminar usuario (admin)             |

### Formato de tareas (`GET /tasks`)

La respuesta es un **árbol**: solo tareas raíz (`parent_id: null`), cada una con subtareas anidadas en `children`.

```json
[
  {
    "id": 1,
    "title": "Comprar",
    "description": null,
    "completed": false,
    "metadata": { "priority": "alta", "due_date": "2026-07-01", "tags": ["casa"] },
    "parent_id": null,
    "children": [
      {
        "id": 2,
        "title": "Leche",
        "parent_id": 1,
        "completed": false,
        "children": []
      }
    ]
  }
]
```

Crear subtarea:

```json
POST /tasks
{ "title": "Leche", "parent_id": 1 }
```

## Acceso por defecto

| Usuario | Contraseña inicial              |
|---------|---------------------------------|
| `admin` | Valor de `ADMIN_INITIAL_PASSWORD` |

Cambia la contraseña del admin en producción y usa un `JWT_SECRET` fuerte.

## Desarrollo

- El backend usa CORS permisivo para facilitar el desarrollo con los clientes.
- Las sesiones en web se guardan en `localStorage`; en móvil, en AsyncStorage; en escritorio, en el directorio de configuración del usuario.
- Los clientes renuevan el access token automáticamente antes de que expire.

## Licencia

Proyecto personal. Añade la licencia que corresponda si lo publicas.
