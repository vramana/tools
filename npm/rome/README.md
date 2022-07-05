# Rome

**Rome** is a formatter, linter, bundler, and [more](https://rome.tools/#development-status) for JavaScript, TypeScript, JSON, HTML, Markdown, and CSS.

**Rome** is designed to replace [Babel](https://babeljs.io/), [ESLint](https://eslint.org/), [webpack](https://webpack.js.org/), [Prettier](https://prettier.io/), [Jest](https://jestjs.io/), and others.

**Rome** unifies functionality that has previously been separate tools. Building upon a shared base allows us to provide a cohesive experience for processing code, displaying errors, parallelizing work, caching, and configuration.

**Rome** has strong conventions and aims to have minimal configuration. Read more about our [project philosophy](https://rome.tools/#philosophy).

**Rome** is [written in Rust](https://rome.tools/blog/2021/09/21/rome-will-be-rewritten-in-rust).

**Rome** has first-class IDE support, with a sophisticated parser that represents the source text in full fidelity
and top-notch error recovery.

**Rome** is [MIT licensed](https://github.com/rome/tools/tree/main/LICENSE) and moderated under the [Contributor Covenant Code of Conduct](https://github.com/rome/tools/tree/main/CODE_OF_CONDUCT.md).


## Installation

```shell
npm i rome@next
```

## Usage

Format files:

```shell
rome format --write ./path ./path/to/file.js
```

For complete documentation, please visit the [official website].


[official website]: https://rome.tools/
