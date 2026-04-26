# localshare

**localshare** is a lightweight file sharing server written in Rust. Run it on any machine on your local network and instantly share files with anyone nearby — no internet connection required.

## ✨ Features

- **🚀 High Performance:** Built on the [Rocket](https://rocket.rs/) web framework with Tokio async IO.
- **📡 Zero-Config Discovery:** Registers itself via mDNS so devices on the same network can find it automatically.
- **📱 QR Code Access:** Displays a QR code in the terminal on startup for instant mobile access.
- **📤 Upload Progress Bar:** Real-time upload progress shown in the browser.
- **📥 Efficient Streaming:** Files are streamed directly from disk — low memory usage even for large files.
- **🗂 Metadata Support:** Track file author and description for every upload.
- **🔐 Optional Authentication:** Protect admin actions (file deletion) behind a password using session-based cookies.
- **🗑 File Deletion:** Admins can delete uploaded files through the web UI or API.
- **🔌 JSON API:** All backend functionality is available programmatically.

---

## 🛠️ Installation & Setup

### Prerequisites

You need **Rust** and **Cargo** installed. Get them from [rustup.rs](https://rustup.rs/).

### Install

```sh
cargo install localshare
```

### Quick Start

**1. Create a new server directory:**

```sh
localshare new my_server
```

This creates `my_server/` with a default `LocalShare.toml` configuration, a `static/` folder containing the web UI, and an `uploads/` folder for received files.

**2. Start the server:**

```sh
localshare run my_server
```

**3. Open the web interface:**

Navigate to `http://<your-local-ip>:8080` in your browser, or scan the QR code printed in the terminal.

---

## 🔐 Authentication (Optional)

By default, localshare runs in open mode — anyone on the network can upload and delete files.

To enable password-protected admin actions, pass `--auth` when creating the server directory:

```sh
localshare new my_server --auth
```

When auth is enabled, you must set the `LOCALSHARE_PASSWORD` environment variable before starting the server:

```sh
LOCALSHARE_PASSWORD=mysecretpassword localshare run my_server
```

If the variable is not set or is empty, the server will refuse to start.

### Admin Login Flow

1. Open the web UI and click **Login as Admin**.
2. If auth is disabled, admin access is granted immediately.
3. If auth is enabled, you are redirected to the login page where you enter the password.
4. Once authenticated, **Delete** buttons appear next to each file.
5. Your session is stored in a secure, encrypted cookie and persists until you close the browser.

---

## 📂 Usage

| Page | URL | Description |
| :--- | :--- | :--- |
| Home | `/` | Lists all uploaded files. Download any file or log in as admin to delete files. |
| Upload | `/upload` | Upload a file with an author name and optional description. Shows a live progress bar. |
| Login | `/login` | Admin login page (only relevant when auth is enabled). |

---

## 🔌 API Reference

All endpoints are available for programmatic access.

### File Endpoints

| Method | Endpoint | Auth Required | Description |
| :--- | :--- | :---: | :--- |
| `GET` | `/api/list` | No | Returns a JSON array of all uploaded file records. |
| `POST` | `/api/upload?author=&filename=&description=` | No | Upload a file as a raw binary body (`application/octet-stream`). Returns `{ "id": "<uuid>" }`. |
| `GET` | `/api/download/<uuid>` | No | Streams the file as a binary attachment. |
| `DELETE` | `/api/delete/<uuid>` | **Yes** | Permanently deletes a file and its metadata. Returns `204 No Content`. |

### Auth Endpoints

| Method | Endpoint | Description |
| :--- | :--- | :--- |
| `GET` | `/api/login?return_url=` | Auth entry point. Redirects to `return_url` if already authenticated or auth is disabled. Redirects to `/login` otherwise. |
| `POST` | `/api/auth` | Submit password (form field: `password`, optional: `from`). Sets a session cookie on success. |
| `GET` | `/api/session` | Returns `200 OK` if the current session is valid, `401 Unauthorized` otherwise. |

### Record Schema (`/api/list`)

```json
[
  {
    "uuid": "550e8400-e29b-41d4-a716-446655440000",
    "name": "photo.jpg",
    "author": "Alice",
    "description": "Holiday photos",
    "uploaded_at": "2025-01-15T10:30:00Z"
  }
]
```

---

## ⚙️ Configuration

The `LocalShare.toml` file is created by `localshare new` and can be edited manually.

```toml
version = "0.2.0"

[app]
port  = "8080"
debug = true
auth  = false   # set to true when using --auth

[path]
db     = "localshare.db"
uploads = "uploads"
static  = "static"
```

---

## 🏗️ Tech Stack

| Layer | Technology |
| :--- | :--- |
| Language | Rust 🦀 |
| Web Framework | [Rocket](https://rocket.rs/) |
| Async Runtime | [Tokio](https://tokio.rs/) |
| Database | SQLite (via rusqlite) |
| Embedded Assets | [rust-embed](https://github.com/pyros2097/rust-embed) |
| mDNS Discovery | [mdns-sd](https://github.com/keepsimple1/mdns-sd) |
| Frontend | HTML5, CSS3, Vanilla JavaScript |

---

## 📄 License

Distributed under the MIT License. See `LICENSE` for more information.