<a name="v0.5.0"></a>
### v0.5.0 (2024-01-12)

#### Breaking Changes

* include_mmd! now always uses CARGO_MANIFEST_DIR as root (PR #47 by [Rjected](https://github.com/Rjected))

#### Miscellaneous

<a name="v0.4.0"></a>
### v0.4.0 (2023-12-13) YANKED

#### Breaking Changes

* `path` attribute is no longer supported for importing diagrams from external files

#### Features

*   `include_mmd!` macro-like syntax for embedding diagrams from files
*   multiple diagrams can now be imported from filesystem per documented entity
*   imported diagrams can now be placed freely at any place inside the doc comment

#### Miscellaneous

*   syn bumped to version 2 (PR #42 by [maurer](https://github.com/maurer)

<a name="v0.3.1"></a>
### v0.3.1 (2023-04-17)

#### Features

*   mermaid is updated to v10 (PR #46 by [frehberg](https://github.com/frehberg))
*   better handling of a failure to load mermaidjs (PR #46 by [frehberg](https://github.com/frehberg))

#### Miscellaneous

*   add Frehberg as a maintainer on GitHub, and package owner on Crates.io

<a name="v0.3.0"></a>
### v0.3.0 (2023-02-16)

#### Maintenance

*   update dependencies

<a name="v0.2.2"></a>
### v0.2.2 (2023-02-02)

#### Bug Fixes

*   gracefully handle failure to write mermaid.js files ([514c67c9](514c67c9))

<a name="v0.2.1"></a>
### v0.2.1 (2023-02-01)

#### Maintenance 

*   MermaidJS updated to version 9.3.0 

<a name="v0.2.0"></a>
## v0.2.0 (2023-01-31)

#### Bug Fixes

*   embedding broken when dependants are built with `--no-deps` [06e263b3](06e263b3) by [frehberg](https://github.com/frehberg)

#### Features

*   allow loading diagrams from filesystem via macro attrs [0eb7e08f](0eb7e08f) by [drbh](https://github.com/drbh)

<a name="v0.1.12"></a>
### v0.1.12 (2022-08-17)

mermaid.js upgraded to version 9.1.4

#### Bug Fixes

*   failing doctest ([680ea555](680ea555))
*   typo in changelog ([75419467](75419467))


<a name="0.1.11"></a>
## 0.1.11 (2021-05-31)


#### Features

*   verbose mermaid.js logging ([33746ab3](33746ab3)) by [yunhong](https://github.com/allenchou13)
*   mermaid.js version 13.4 ([33746ab3](33746ab3)) by [yunhong](https://github.com/allenchou13)

<a name="0.1.10"></a>
## 0.1.10 (2021-05-31)


#### Features

*   lower MSRV to 1.31.1 ([2fd0f032](2fd0f032))

<a name="0.1.9"></a>
## 0.1.9 (2021-05-15)

#### Features

*   upgrade mermaid.js to 8.10.1 ([fbb13e1db](fbb13e1db)) by [José Duarte](https://github.com/jmg-duarte)

<a name="0.1.8"></a>
## 0.1.8 (2021-04-08)

#### Bug Fixes

*   fallback to CDN version of mermaid.js if local isn't found ([de9f274e](de9f274e))

<a name="0.1.7"></a>
## 0.1.7 (2021-04-08)

#### Features

*   use local version of the mermaid.js library ([8f523072](8f523072)) by [Le Savon Fou](https://github.com/lesavonfou)

#### Bug Fixes

*   fix doctests ([ea685563](ea685563)) by [Le Savon Fou](https://github.com/lesavonfou)

<a name="0.1.6"></a>
## 0.1.6 (2021-01-28)


#### Bug Fixes

*   use regex to detect the dark themes reliably on docs.rs ([ce24cd6e](ce24cd6e))


<a name="0.1.5"></a>
## 0.1.5 (2021-01-28)


#### Bug Fixes

*   initialization script wasn't firing at page load ([36268718](36268718))


<a name="0.1.4"></a>
## 0.1.4 (2021-01-28)


#### Features

*   dark mode and custom themes ([62ec6783](62ec6783))
*   add crossorigin attribute to script tag ([fa9f4546](fa9f4546)) by [Mark Schmale](https://github.com/themasch)
