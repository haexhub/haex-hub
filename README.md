# 🧩 HaexHub – The European “Everything App”

## 🌍 Vision

We are living in the **computer age** — nearly everyone owns multiple devices: a smartphone, a laptop, perhaps even a desktop PC or tablet.  
Each of these runs its own **operating system** — Windows, macOS, Linux, Android, iOS — and hosts a unique mix of **apps and data**.

Unfortunately, **interoperability** between these devices is often poor or even impossible.  
The reasons are many:

- **Platform lock-in**: Vendors like Microsoft, Apple, or Google design systems that make it easy to _enter_ their ecosystem but difficult to _leave_.
- **Fragmented software development**: Developers face high technical and financial hurdles to support multiple platforms at once.

Creating and maintaining one secure, high-quality app for _all_ systems can be almost impossible — especially for small teams, startups, and indie developers.

And then there’s **distribution**: each platform requires its own build, packaging, signing, and publishing process.  
What if you could build your app **once** and deploy it **everywhere**?

> **HaexHub** makes that possible — giving every web app or PWA **superpowers**.

With HaexHub, developers can extend functionality via **extensions** that run securely inside the app, with carefully controlled permissions for accessing system features (files, shell, database, etc.).

---

## 🚀 Enter HaexHub

HaexHub provides a **framework** for building and running modular, sandboxed **web extensions** — web apps that run in an isolated environment but can communicate securely with the host.

Each extension:

- Runs inside an **IFrame**.
- Uses **postMessage APIs** to communicate with HaexHub.
- Declares required **permissions** in a manifest file.
- Can be added or removed at runtime.

Without explicit permission, extensions cannot access the file system, network, or external resources — ensuring **privacy and security** by default.  
Once granted, however, extensions can unlock full desktop-like capabilities:  
access files, execute commands, or interact with SQLite databases.

Imagine a **web-based VS Code** that can directly access your local shell and file system — something that current web IDEs can’t do.  
With HaexHub’s permission model, such power is possible, but **always under user control**.

HaexHub itself is **cross-platform** and runs on:

- 💻 Windows, macOS, Linux
- 📱 Android, iOS
- 🧠 Desktops, laptops, tablets, smartphones

All user and extension data is stored in a **locally encrypted SQLite database**.  
To sync across devices, HaexHub can connect to a **synchronization server** — which you can even **self-host** for maximum independence.

> 🛡️ HaexHub is built on the principles of **privacy, security, and digital sovereignty**.

The user is always in control of their data — deciding what to share, and with whom.

---

## 🧠 Technical Foundations

HaexHub is powered by **[Tauri](https://v2.tauri.app/)** — a secure, efficient framework for building native apps from web technologies.

Unlike Electron (used by apps like VS Code), Tauri:

- Uses **native rendering engines** (WebView2, WKWebView, WebKitGTK)
- Produces **smaller, faster apps**
- Enforces **strong sandboxing and permission models**

HaexHub builds upon Tauri’s security features, extending them to third-party extensions.

### 🏡 Local-first by Design

HaexHub follows a **strict local-first architecture**:

- Works **offline** without accounts or internet.
- Stores data locally in **encrypted SQLite**.
- Uses **CRDTs (Conflict-free Replicated Data Types)** for safe synchronization across devices — even with encrypted data.

Unlike many “local-first” apps, HaexHub doesn’t just cache data in the browser.  
Your data truly resides **on your disk**, not under a browser’s limited storage policy.

Optionally, HaexHub can sync databases via a backend service — self-hosted or external — with optional **end-to-end encryption**.

---

## 🧩 Extensions

Extensions are the heart of HaexHub.

Everything the user interacts with — from password management to file syncing — will be implemented as **extensions**.

There are two types:

- **Official/Core Extensions**
- **Third-Party Extensions**

Each extension is a **web app** bundled via your preferred frontend stack:

> Vue, React, Svelte, Angular, Vite, Webpack, Rollup — you name it.

### 🔐 Example: Password Manager

A first official extension will be a **Password Manager**, built with **Vue/Nuxt**:

- Declares database permissions via its manifest.
- Manages login credentials locally in encrypted SQLite.
- Can tag entries (e.g. “Email”) for use by other extensions — such as an email client.

### 🗂 Example: File Synchronization

Another planned core extension will handle **file synchronization**:

- Syncs files/folders between devices and cloud providers (e.g. S3, Google Drive, Dropbox).
- Lets users define sync rules per device.
- Stores configuration securely in the local database.

### 💬 Future Extensions

- Calendar & Contacts
- Collaborative document management
- Messenger
- Browser & Payment Services (e.g., GNU Taler integration)

With this modular design, HaexHub can evolve into a true **European alternative to WeChat** — but open, federated, and privacy-first.

---

## 🧰 Installation & Setup

### 📦 Prerequisites

Install the following dependencies:

- [Node.js / nvm](https://nodejs.org/en/download)
- [Tauri](https://v2.tauri.app/start/prerequisites/)
- [Rust](https://v2.tauri.app/start/prerequisites/#rust)
- [Android Studio](https://developer.android.com/studio?hl=de)
- WebKit2GTK + GTK3

#### 🐧 Debian / Ubuntu

```bash
sudo apt update
sudo apt install \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

#### 🦊 Fedora

```bash
sudo dnf install \
  webkit2gtk4.1-devel \
  gtk3-devel \
  libappindicator-gtk3 \
  librsvg2-devel
```

#### ⚙️ Development

Make sure port 3003 is available (or adjust it in `nuxt.config.ts` and `src-tauri/tauri.conf.json`).

```bash
git clone https://github.com/haexhub/haex-vault.git
cd haex-vault
pnpm install
pnpm tauri dev
```

#### 🧭 Summary

HaexHub aims to:

- Simplify cross-platform app development
- Empower users with local-first privacy
- Enable developers to create modular, permissioned extensions
- Bridge the gap between web and native worlds

HaexHub is the foundation for a decentralized, privacy-friendly, European “everything app.”
