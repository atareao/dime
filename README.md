<p align="center">
  <img src="https://raw.githubusercontent.com/PKief/vscode-material-icon-theme/ec559a9f6bfd399b82bb44393651661b08aaf7ba/icons/folder-markdown-open.svg" width="100" alt="project-logo">
</p>
<p align="center">
    <h1 align="center">DIME</h1>
</p>
<p align="center">
    <em>Empowering conversation with intelligent tech interactions.</em>
</p>
<p align="center">
	<img src="https://img.shields.io/github/license/atareao/dime?style=default&logo=opensourceinitiative&logoColor=white&color=0080ff" alt="license">
	<img src="https://img.shields.io/github/last-commit/atareao/dime?style=default&logo=git&logoColor=white&color=0080ff" alt="last-commit">
	<img src="https://img.shields.io/github/languages/top/atareao/dime?style=default&color=0080ff" alt="repo-top-language">
	<img src="https://img.shields.io/github/languages/count/atareao/dime?style=default&color=0080ff" alt="repo-language-count">
<p>
<p align="center">
	<!-- default option, no dependency badges. -->
</p>

<br><!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary><br>

- [ Overview](#-overview)
- [ Features](#-features)
- [ Repository Structure](#-repository-structure)
- [ Modules](#-modules)
- [ Getting Started](#-getting-started)
  - [ Installation](#-installation)
  - [ Usage](#-usage)
  - [ Tests](#-tests)
- [ Project Roadmap](#-project-roadmap)
- [ Contributing](#-contributing)
- [ License](#-license)
- [ Acknowledgments](#-acknowledgments)
</details>
<hr>

##  Overview

The Dime project is a chat program powered by an intelligent IA Bot. It leverages a configuration file (dime.yml) for customizable settings and interacts with a chat completion API for responses. The projects core functionalities include user input processing, configuration initialization, request handling, error management, and token management. By combining user interaction with AI capabilities, Dime offers a dynamic and engaging chat experience for users.

---

##  Features

|    |   Feature         | Description |
|----|-------------------|---------------------------------------------------------------|
| ⚙️  | **Architecture**  | The project follows a modular architecture with a focus on Rust programming language. It leverages async IO, HTTP handling, JSON/YAML serialization, and logging for robust functionality. Configuration settings are defined in YAML format.|
| 🔩 | **Code Quality**  | The codebase maintains a high standard of quality with clear structure and adherence to Rust best practices. It utilizes libraries for directory operations, async IO, HTTP handling, JSON/YAML serialization, logging, and command-line parsing.|
| 📄 | **Documentation** | The project provides detailed documentation including configuration parameters, dependencies, and how to interact with the IA Bot. Documentation covers setup instructions and usage guidelines for developers.|
| 🔌 | **Integrations**  | Key integrations include lock, rs, rust, yml, toml for configuration and dependency management. The project also interacts with external APIs for chat completion and response handling.|
| 🧩 | **Modularity**    | The codebase exhibits strong modularity allowing for easy maintenance and reusability of components. Modules such as IA settings, request handling, error management, and file I/O are well-organized.|
| 🧪 | **Testing**       | Testing frameworks and tools used in the project are not explicitly mentioned but can be integrated for ensuring code stability and functionality.|
| ⚡️  | **Performance**   | The project's efficiency is optimized through the use of async IO and other performance-oriented libraries. Resource usage is well managed for smooth operation of the IA Bot.|
| 🛡️ | **Security**      | Data protection and access control are ensured through secure handling of configurations and external interactions. Token management and error handling mechanisms enhance security measures.|
| 📦 | **Dependencies**  | Key dependencies include libraries for directory operations, async IO, HTTP handling, JSON/YAML serialization, logging, and command-line parsing. These dependencies enhance the project's functionality.|

---

##  Repository Structure

```sh
└── dime/
    ├── Cargo.lock
    ├── Cargo.toml
    ├── dime.yml
    └── src
        ├── ia.rs
        └── main.rs
```

---

##  Modules

<details closed><summary>.</summary>

| File                                                                 | Summary                                                                                                                                                                                                                                                              |
| ---                                                                  | ---                                                                                                                                                                                                                                                                  |
| [dime.yml](https://github.com/atareao/dime/blob/master/dime.yml)     | Defines configuration parameters for the Dime project, including log level, API base URL, endpoints, and model details.                                                                                                                                              |
| [Cargo.toml](https://github.com/atareao/dime/blob/master/Cargo.toml) | Defines dependencies and version specifications for the dime project. Utilizes libraries for directory operations, async IO, HTTP handling, JSON/YAML serialization, logging, and command-line parsing. Enables robust functionality for the project's architecture. |

</details>

<details closed><summary>src</summary>

| File                                                               | Summary                                                                                                                                                                                                                     |
| ---                                                                | ---                                                                                                                                                                                                                         |
| [main.rs](https://github.com/atareao/dime/blob/master/src/main.rs) | Implements a chat program with IA internals. Retrieves config settings from various paths. Initializes logging based on the config. Accepts user input to interact with IA for responses.                                   |
| [ia.rs](https://github.com/atareao/dime/blob/master/src/ia.rs)     | Defines IA Bot settings, request handling, error management, and file I/O. Facilitates token management, default config creation, and reading. Core features include querying a chat completion API and handling responses. |

</details>

---

##  Getting Started

**System Requirements:**

* **Rust**: `version x.y.z`

###  Installation

<h4>From <code>source</code></h4>

> 1. Clone the dime repository:
>
> ```console
> $ git clone https://github.com/atareao/dime
> ```
>
> 2. Change to the project directory:
> ```console
> $ cd dime
> ```
>
> 3. Install the dependencies:
> ```console
> $ cargo build
> ```

###  Usage

<h4>From <code>source</code></h4>

> Run dime using the command below:
> ```console
> $ cargo run
> ```

###  Tests

> Run the test suite using the command below:
> ```console
> $ cargo test
> ```

---

##  Project Roadmap

- [X] `► INSERT-TASK-1`
- [ ] `► INSERT-TASK-2`
- [ ] `► ...`

---

##  Contributing

Contributions are welcome! Here are several ways you can contribute:

- **[Report Issues](https://github.com/atareao/dime/issues)**: Submit bugs found or log feature requests for the `dime` project.
- **[Submit Pull Requests](https://github.com/atareao/dime/blob/main/CONTRIBUTING.md)**: Review open PRs, and submit your own PRs.
- **[Join the Discussions](https://github.com/atareao/dime/discussions)**: Share your insights, provide feedback, or ask questions.

<details closed>
<summary>Contributing Guidelines</summary>

1. **Fork the Repository**: Start by forking the project repository to your github account.
2. **Clone Locally**: Clone the forked repository to your local machine using a git client.
   ```sh
   git clone https://github.com/atareao/dime
   ```
3. **Create a New Branch**: Always work on a new branch, giving it a descriptive name.
   ```sh
   git checkout -b new-feature-x
   ```
4. **Make Your Changes**: Develop and test your changes locally.
5. **Commit Your Changes**: Commit with a clear message describing your updates.
   ```sh
   git commit -m 'Implemented new feature x.'
   ```
6. **Push to github**: Push the changes to your forked repository.
   ```sh
   git push origin new-feature-x
   ```
7. **Submit a Pull Request**: Create a PR against the original project repository. Clearly describe the changes and their motivations.
8. **Review**: Once your PR is reviewed and approved, it will be merged into the main branch. Congratulations on your contribution!
</details>

<details closed>
<summary>Contributor Graph</summary>
<br>
<p align="center">
   <a href="https://github.com{/atareao/dime/}graphs/contributors">
      <img src="https://contrib.rocks/image?repo=atareao/dime">
   </a>
</p>
</details>

---

##  License

This project is protected under the [SELECT-A-LICENSE](https://choosealicense.com/licenses) License. For more details, refer to the [LICENSE](https://choosealicense.com/licenses/) file.

---

##  Acknowledgments

- List any resources, contributors, inspiration, etc. here.

[**Return**](#-overview)

---
