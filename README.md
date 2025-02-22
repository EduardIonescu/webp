# CLI WebP Converter

This Rust application recursively converts images within a specified directory (or a single image) to WebP format. 

## Features

-   **Recursive Directory Conversion:** Processes all images within a directory and its subdirectories.
-   **Quality Control:** Adjust the WebP encoding quality from 0 to 100.
-   **Lossless Compression:** Enable lossless compression for optimal image quality.
-   **Encoding Method Selection:** Choose the encoding method for WebP conversion.
-   **Directory Traversal Depth:** Limit the depth of directory traversal.
-   **Original Image Preservation:** Option to keep the original image if it results in a smaller file size.
-   **Multi-threading Support:** Uses `rayon` for parallel image processing, improving speed.
-   **Detailed Summary:** Provides summary of the process, including input size, output size, size reduction, and duration.
-   **Clear Logging:** Displays progress and conversion results in a formatted table.

## Installation

1.  **Install Rust and Cargo:** If you don't have Rust installed, download and install it from [rustup.rs](https://rustup.rs/).
2.  **Clone the Repository:**

    ```bash
    git clone [<your-repository-url>](https://github.com/EduardIonescu/webp.git)
    cd webp
    ```

3.  **Build the Application:**

    ```bash
    cargo build --release
    ```

    The executable will be located in `target/release/`.

    ## Usage

    ```bash
    ./target/release/webp [OPTIONS] --input <INPUT_PATH>
    ```

## TODO
- [ ] Resizing
- [ ] Show the errors better in the summary
