# **localshare**

**localshare** is a lightweight, high-performance file sharing server written in Rust. It allows users to easily share files over a local network with a simple web interface and efficient streaming capabilities.

**Note:** This project recently underwent a complete rewrite to improve performance and code quality.

## **✨ Features**

* **🚀 High Performance:** Built on the [Rocket](https://rocket.rs/) web framework and Tokio.  
* **Efficient Streaming:** Files are streamed directly from disk to the client using asynchronous IO, ensuring low memory usage even for large files.  
* **Metadata Support:** Track file ownership (Author) and descriptions.  
* **Simple UI:** Clean HTML/CSS interface for listing and uploading files.  
* **JSON API:** Backend endpoints available for programmatic access.

## **🛠️ Installation & Setup**

### **Prerequisites**

You need **Rust** and **Cargo** installed on your machine. If you don't have them, install them via [rustup.rs](https://rustup.rs/).

### **Install and Run**

1. **Install with cargo:**  
    `cargo install localshare`

2. **Create new server configuration:**  
    `localshare new "my_server"`


3. **Run & Access the application:**
   Run with: `localshare run "my_server"`
   Open your browser and navigate to:  
   http://<your_local_ip>:<your_configured_port>

## **📂 Usage**

1. **Home Page (/):** View the list of all uploaded files. Click "Download" to stream a file or "Upload New File" to add content.  
2. **Upload Page (/upload):** Enter your name (Author), a description, and select a file to upload.  
3. **Storage:** Uploaded files are stored in the uploads/ directory in the project root.

## **🔌 API Documentation**

If you want to interact with localshare programmatically, you can use the following endpoints:

| Method | Endpoint | Description |
| :--- | :--- | :--- |
| GET | /api/list | Returns a JSON list of all available files and metadata. |
| GET | /api/download/\<uuid\> | Streams the file content as a binary attachment. |
| POST | /api/upload | Upload a file. Requires query params: author, description, filename. |

## **🏗️ Tech Stack**

* **Language:** Rust 🦀  
* **Web Framework:** Rocket  
* **Async Runtime:** Tokio  
* **Frontend:** HTML5, CSS3, Vanilla JavaScript


## **📄 License**

Distributed under the MIT License. See LICENSE for more information.
