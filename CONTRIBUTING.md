# Contributing

## Prerequisites

The following software is required:

* [Rust](https://www.rust-lang.org/tools/install)

  If Rust is already installed, please run `rustup update` to make sure you're up to date.

The following software is recommended:

* [Visual Studio Code](https://code.visualstudio.com/)

  When opening this project directory and prompted, please install recommended extensions.
  These are limited only to things that help keep the project clean and should not feel intrusive.

* [WiX 3.11](https://wixtoolset.org/releases/)

  WiX 4 is not supported at this time, but you should be able to install both side by side.

## Testing

To build the Windows Installer package (MSI) for debugging, run the following commands:

```powershell
msbuild -t:rebuild examples/example.wixproj
msiexec /i $PWD\target\debug\example.msi /l*v install.log
```
