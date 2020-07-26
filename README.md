# Greasy

<!-- PROJECT SHIELDS -->
<!--
*** I'm using markdown "reference style" links for readability.
*** Reference links are enclosed in brackets [ ] instead of parentheses ( ).
*** See the bottom of this document for the declaration of the reference variables
*** for contributors-url, forks-url, etc. This is an optional, concise syntax you may use.
*** https://www.markdownguide.org/basic-syntax/#reference-style-links
-->
[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![MIT License][license-shield]][license-url]



<!-- PROJECT LOGO -->
<br />
<p align="center">
  <a href="https://github.com/r4gus/greasy">
    <img src="images/greasy.png" alt="Logo" width="160" height="80">
  </a>

  <h3 align="center">Greasy</h3>

  <p align="center">
    A general FAT file system information and data recovery tool written in Rust. 
    <br />
    <a href="https://github.com/r4gus/greasy"><strong>Explore the docs »</strong></a>
    <br />
    <br />
    <a href="https://github.com/r4gus/greasy/issues">Report Bug</a>
    ·
    <a href="https://github.com/r4gus/greasy/issues">Request Feature</a>
  </p>
</p>



<!-- TABLE OF CONTENTS -->
## Table of Contents

* [About the Project](#about-the-project)
  * [Built With](#built-with)
* [Getting Started](#getting-started)
  * [Prerequisites](#prerequisites)
  * [Installation](#installation)
* [Usage](#usage)
* [Roadmap](#roadmap)
* [Contributing](#contributing)
* [License](#license)
* [Contact](#contact)
* [Acknowledgements](#acknowledgements)



<!-- ABOUT THE PROJECT -->
## About The Project

__Greasy__ can currently display the details associated with a FAT16/ 32 file system. One can look up general information
like the cluster size (in Bytes) or the file system layout. It is also possible to list contents of directories in a tree-like format.

### Built With
* [Rust](https://www.rust-lang.org) 

<!-- GETTING STARTED -->
## Getting Started

To get a local copy up and running follow these simple example steps.

### Prerequisites

First you need to install Rust. 

#### Linux
On Linux you can just run the following in the command line and then follow the instructions:
```Bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### Windows
For Windows see: [other Rust installation methods](https://forge.rust-lang.org/infra/other-installation-methods.html).


### Installation

1. Clone the repo
```sh
git clone https://github.com/r4gus/greasy.git
```

2. Switch into the folder and run
```sh
cargo run <FILE>
```

The program expects a FAT16/ 32 volume as first command line argument.


<!-- USAGE EXAMPLES -->
## Usage

To display a help text run the programm with __--help__.
```Bash
Greasy 0.1.0
David Sugar (r4gus)
Fat file system information and data recovery tool

USAGE:
    greasy [FLAGS] <INPUT>

FLAGS:
    -h, --help       Prints help information
    -i, --info       Display general file system layout information
    -t, --tree       Display all directories in a tree like manner
    -V, --version    Prints version information

ARGS:
    <INPUT>    Fat volume to parse (e.g. fat-16.dd)
```

You can display some general file system information with the -i or --info option.
```Bash
cargo run -i fat-16.dd

FILE SYSTEM INFORMATION
--------------------------------
File System Type: FAT16   
OEM Name: mkfs.fat
Vloume ID:
Volume Label (Boot Sector):
File System Type Label: FAT16   

Size
--------------------------------
Sector Size (in bytes): 512
Cluster Size (in bytes): 4096
Cluster Range: 2 - 65468

File System Layout (in sectors)
--------------------------------
Total Sector Range: 0 - 524287
|- Reserved: 0 - 7
|  └─ Boot Sector: 0
|- FAT 0: 8 - 263
|- FAT 1: 264 - 519
└─ Data Area: 520 - 524287
    |- Root: 520 - 551
    └─ Cluster Area: 552 - 524287
```

You can display the folder structure in a tree like manner with the -t or -tree option.
```Bash
File layout:
Deleted = X, Disk Volume = V
Directory = D, File = F
---------------------------------------
*[Alice: V]
*[Work: D]
**[school.tar.gz: F]
*[Pictures: D]
**[friend and me in paris.jpg: F]
**[my dog.jpg: F]
**[alice.jpg: F]
**[�ORK    JPG: X | F]
*[Documents: D]
**[DOS-Partition - 1.jpg: F]
**[�HANNO~1PNG: X | F]
**[groceries.md: F]
**[shannon.png: F]
**[�ERD_T~2SWP: X | F]
*[System Volume Information: D]
**[ClientRecoveryPasswordRotation: D]
**[AadRecoveryPasswordDelete: D]
**[WPSettings.dat: F]
**[IndexerVolumeGuid: F]
```

<!-- ROADMAP -->
## Roadmap

See the [open issues](https://github.com/r4gus/greasy/issues) for a list of proposed features (and known issues).


## Contributing

Contributions are what make the open source community such an amazing place to be learn, inspire, and create. Any contributions you make are **greatly appreciated**.

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request


LICENSE
## License

Distributed under the MIT License. See `LICENSE` for more information.


<!-- CONTACT -->
## Contact

Project Link: [https://github.com/r4gus/greasy](https://github.com/r4gus/greasy)



<!-- ACKNOWLEDGEMENTS -->
## Acknowledgements
* [GitHub Emoji Cheat Sheet](https://www.webpagefx.com/tools/emoji-cheat-sheet)
* [Img Shields](https://shields.io)
* [Choose an Open Source License](https://choosealicense.com)
* [GitHub Pages](https://pages.github.com)
* [Animate.css](https://daneden.github.io/animate.css)
* [Loaders.css](https://connoratherton.com/loaders)
* [Slick Carousel](https://kenwheeler.github.io/slick)
* [Smooth Scroll](https://github.com/cferdinandi/smooth-scroll)
* [Sticky Kit](http://leafo.net/sticky-kit)
* [JVectorMap](http://jvectormap.com)
* [Font Awesome](https://fontawesome.com)





<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->

[contributors-shield]: https://img.shields.io/github/contributors/r4gus/greasy?style=flat-square
[contributors-url]: https://github.com/r4gus/greasy/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/r4gus/greasy?style=flat-square
[forks-url]: https://github.com/r4gus/greasy/network
[stars-shield]: https://img.shields.io/github/stars/r4gus/greasy?style=flat-square
[stars-url]: https://github.com/r4gus/greasy/stargazers
[issues-shield]: https://img.shields.io/github/issues/r4gus/greasy?style=flat-square
[issues-url]: https://github.com/r4gus/greasy/issues
[license-shield]: https://img.shields.io/github/license/r4gus/greasy?style=flat-square
[license-url]: https://github.com/r4gus/greasy/blob/traits/LICENSE.txt
[product-screenshot]: images/screenshot.png

