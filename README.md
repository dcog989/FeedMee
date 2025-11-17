# FeedMee PoC (Tauri + Svelte)

Proof-of-Concept for the FeedMee RSS Reader application.

## Structure

- 

## Prerequisites

Before you begin, ensure you have the following installed:

1. **Rust**: Follow the instructions at [rust-lang.org](https://www.rust-lang.org/tools/install).
2. **Node.js**: LTS version recommended. You can download it from [nodejs.org](https://nodejs.org/).
3. **Tauri's System Dependencies**: Follow the setup guide for your specific operating system on the [Tauri website](https://tauri.app/v1/guides/getting-started/prerequisites). This is a critical step that includes build tools like C++ compilers and webview libraries.

## How to Run

1. **Install Root Dependencies (Tauri CLI):**
    Open a terminal in the project root directory and run:

    ```bash
    npm install
    ```

2. **Install Frontend Dependencies:**
    Navigate to the frontend directory and install its dependencies:

    ```bash
    cd frontend
    npm install
    cd ..
    ```

3. **Run in Development Mode:**
    From the project root directory, run the following command. This will launch the application in a development window with hot-reloading for both the frontend and backend.

    ```bash
    npm run tauri dev
    ```

## How to Build

1. **Ensure all dependencies are installed** by following steps 1 and 2 from the "How to Run" section.

2. **Build the Application:**
    From the project root directory, run the build command:

    ```bash
    npm run tauri build
    ```

    This will compile the Rust backend in release mode and bundle it with the frontend into a standalone, native executable. The final installer or application file will be located in `src/target/release/bundle/`.
