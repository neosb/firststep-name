# First Step Name

This application is designed to check username availability across social networks. Our goal is to provide a tool for users to quickly determine if their desired username is already taken on popular social media platforms.

## About This Project

First Step Name is a command-line tool and a web application built with Rust. It leverages crates like `reqwest`, `tokio`, `clap`, `serde`, `poem`, `askama`, `chrono`, `url`, and `futures-util`. It aims to efficiently query multiple websites to check username availability.

### Core Functionalities

This project offers a range of features that enable it to perform its magic:

*   **Username Availability Check:** Checks if a given username is available on a predefined list of social media platforms.
*   **Multi-platform Support:** Utilizes a JSON file (`social_sites.json`) to define which platforms to check, allowing for easy expansion and customization.
*   **Concurrency:** Can utilize multiple threads to speed up the checking process.
*   **Output Formats:** Supports outputting results in plain text (`txt`), JSON, or via a web interface.
*   **Web Interface:** Provides a dynamic web UI for checking usernames and viewing results in real-time.
*   **Data Download:** Option to download the latest platform data from a GitHub repository.

### How It Works (The Magic Behind the Scenes)

At its core, First Step Name employs an asynchronous, multi-threaded approach to check username availability. Here's a simplified overview of the process:

1.  **Input Processing:** Parses command-line arguments or accepts input from the web UI to get the target username.
2.  **Data Loading:** Reads site information (URLs, expected response codes/strings for taken/available states) from `social_sites.json`.
3.  **Concurrent Checking:** For each social site, it constructs the appropriate URL with the username and sends an HTTP GET request. These requests are managed concurrently using `tokio` tasks, respecting the specified thread count.
4.  **Response Analysis:** Each response is analyzed based on its HTTP status code and body content, comparing it against the `e_code`, `e_string`, `m_code`, and `m_string` defined in the `SiteData`.
5.  **Result Aggregation:** Results (site name, status, URL, logo URL, and any errors) are collected.
6.  **Output Generation:** Results are presented to the user via the console (txt), saved to a JSON file, or displayed dynamically on the web interface.

We leverage asynchronous HTTP requests with `reqwest` and `tokio` for efficient and concurrent checking, and `poem` for the web server functionality. `askama` is used for server-side HTML templating.

## Getting Started

To get up and running with First Step Name, please follow these steps:

Skip to number 4. if you're not interested in the code

1.  **Prerequisites:** Ensure you have Rust and Cargo installed. You can find instructions on the [official Rust website](https://www.rust-lang.org/tools/install).
2.  **Cloning the Repository:**
    ```bash
    git clone https://github.com/NutekSecurity/firststep-name.git
    cd firststep-name
    ```
3.  **Building the Project:**
    ```bash
    cargo build
    ```
4.  **Running the Application:**

    *   **Command-line:**
        ```bash
        cargo install firststep-name
        firststep-name <username> [options]
        # Example:
        # cargo install firststep-name
        # firststep-name johndoe -o txt -t 10
        # firststep-name janesmith --download
        ```
        Refer to `firststep-name --help` for all available options.

    *   **Web Server:**
        ```bash
        firststep-name --output web --username joasia_chmiel
        ```
        The web server will be available at `http://127.0.0.1:3003`.

5. You can also get the latest working version on GitHub releses page.

6.  **Configuration:**
    The `social_sites.json` file in the project root contains the data for sites to check. You can modify this file to add or remove platforms. If the file is missing, the application will attempt to download it from GitHub.

## Contributing

We are passionate about building a robust and valuable application, and we believe that collaboration is key to achieving this. We enthusiastically welcome contributions from the community!

If you're interested in helping maintain and expand this codebase, here are a few ways you can get involved:

*   **Reporting Bugs:** If you encounter any issues, please open a detailed bug report in the [GitHub Issues](https://github.com/NutekSecurity/firststep-name/issues).
*   **Suggesting Features:** Have an idea for a new feature or an improvement? We'd love to hear it! You can open an issue to suggest new features.
*   **Submitting Pull Requests:** If you'd like to contribute code, please fork the repository and submit a pull request with your changes. Ensure your code adheres to our coding standards and includes relevant tests. We appreciate clear commit messages and concise pull request descriptions.
*   **Documentation:** Clear and comprehensive documentation is crucial. If you find any areas that could be improved, your contributions are highly valued. Feel free to submit a pull request with documentation updates.

Please refer to our [CONTRIBUTING.md](CONTRIBUTING.md) for more detailed guidelines on contributing.

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.
