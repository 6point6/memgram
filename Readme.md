
<h1 align="center">Memgram</h1> 
<p align="center">A CLI binary analysis tool</p>

<p align="center">
  <a href="#about">About</a> •
  <a href="#key-features">Key Features</a> •
  <a href="#grammar-format">Grammar Format</a> •
  <a href="#installation">Installation</a> •
  <a href="#usage-examples">How To Use</a> •
  <a href="#limitations">Limitations</a> 
</p>

# About

`memgram` has been developed to aid with reverse engineering unknown file formats and memory structures. Custom data structures found when reverse engineering can be quickly described in an easily readable TOML like format called a grammar. `memgram` applies these grammars to custom data structures in files and displays formatted prettified output of data.

`memgram` is heavily inspired by the hex editors:
* [Synalyze It!](https://www.synalysis.net/)
* [010](https://www.sweetscape.com/010editor/)

## Key Features

### Grammars

`memgram` utilises easily readable file format descriptions called grammars.

<img src="https://github.com/6point6/memgram/blob/master/images/grammar_example.png" width="640" />

### Coloured Formatted Output

If supported display types are specified in a grammar, the "Formatted Data" coloumn will display the formatted data.
The current supported types are:

* hexle - Display data in little endian hex string format
* ascii - Display data in ASCII format
* ipv4be - Display data in IPv4 big endian format
* ipv4le - Display data in IPv4 little endian format
* utf16be - Display data in UTF16 big endian format
* utf16le - Display data in UTF16 little endian format
* x86_32 - Display x86_32 assembly format)

If a display type not listed above is used, `memgram` will default to formating data as a hex string in native endianess. The endianess of this default format can be changed without affecting the supported types in both the table and hex views using the `-e` and `-E` flags respectively.

<img src="https://github.com/6point6/memgram/blob/master/images/test_format.png" width="640" />

### C Struct Support
C structs can be converted to the grammar format, with support for C basic types. Due to the fact that C basic types may have different sizes depending on the system the code is compiled on, the the most common size for each type has been selected (e.g short is 2 bytes). The structs can either be converted to a grammar file or used directly to display data with the option of reversing endianess.

<img src="https://github.com/6point6/memgram/blob/master/images/c_struct_example.png" width="640" />

### Multipliying Field Entries
To save typing it's possible to multiply an entry x number of times, for example below is the entire grammar for `Master Boot Record` structure:

```toml
[metadata]
    name = 'MBR'

[[fields]]
    name = "Bootstrap Code"
    size = 0x1BE
    data_type = "mixed"
    display_format = 'hex'
    description = 'MBR bootstrap code'

[[fields]] * 4
    name = "Partition Entry"
    size = 0x10
    data_type = "Partition entry"
    display_format = 'hex'
    description = 'MBR partition entry'

[[fields]]
    name = "Boot signature"
    size = 0x02
    data_type = "Signature"
    display_format = 'hex'
    description = 'MBR boot signature'
```

## Grammar Format

Grammars describe the data `memgram` reads, formats and displays. Grammars are written in a TOML like format, infact the format is almost identical apart from a few hacks to the syntax.

Each grammar file starts with what is referred to in TOML syntax as a [Table](https://github.com/toml-lang/toml#user-content-table). The first Table in a grammar file is allways `[metadata]` and holds a single key value pair. The key is allways `name`, and the user can fill in the value with what they wish to name the data structure, e.g `name = MBR`. 

An example is shown below:
```toml
[metadata]
    name = 'MBR'
```

Following this a series of what is referred to in TOML as an [Array of tables](https://github.com/toml-lang/toml#user-content-table). Each entry contains data describing a single field in the data structure.

Listing all of the keys in an entry is mandatory. A list of keys and description of their potential values is shown below:

* The `name` key value is the name of your field (TOML String)
* The `size` key value is how large the field is in bytes  (TOML Integer)
* The `data_type` key value the name for your data type (TOML String)
* The `display_format` key value is a supported format or a custom format (TOML String)
* The `description` key value is the description of the field (TOML String)

An entry example:
```toml
[[fields]]
    name = "Bootstrap Code"
    size = 0x1BE
    data_type = "mixed"
    display_format = 'hex'
    description = 'MBR bootstrap code'
```

Additionally, specifiying a multiplier for an entry is possible. A multiplier tells `memgram` to repeat an entry a certain number of times. For example, when creating a grammar describing the `Master Boot Record` structure, instead of creating four different entries for four parition entries, we can multiply a single partition entry four times:

```toml
[[fields]] * 4
    name = "Partition Entry"
    size = 0x10
    data_type = "Partition entry"
    display_format = 'hex'
    description = 'MBR partition entry'
```

A multiplier is specfied by adding `* x` of after a `[[fields]]` entry where x is the number of times you wish to repeat the entry.

## Installation

`memgram` can be run on the following platforms:
* Linux
* macOS
* Windows

### Standalone Binaries

Binarys for each platform can be found here: https://github.com/6point6/memgram/releases

### Compiling From Source

If you wish to compile from source:
1. Follow instructions for installing rust: https://www.rust-lang.org/tools/install
2. `git clone https://github.com/6point6/memgram.git`
3. `cd memgram`
4. `cargo build --release`

The binary can then be found in `./target/release/`

### Installing Binary

1. Follow instructions for installing rust: https://www.rust-lang.org/tools/install
2. `git clone https://github.com/6point6/memgram.git`
3. `cd memgram`
4. `cargo install --path .`

## Usage Examples

* Display formatted data at starting at offset 0 into mbr.bin based on the mbr.toml grammar:
  * `memgram -g grammar/mbr.toml -b examples/mbr.bin`
* Display description table and formatted data:
  * `memgram -g grammar/mbr.toml -b examples/mbr.bin -d`
* Convert C struct `COFFHeader.h` to grammar file `coff_header.toml` :
  * `memgram -c examples/COFFHeader.h -o grammar/coff_header.toml`
* Use C struct COFFHeader to format data in .exe starting at offset 244 and reverse both table and hex view endianess:
  * `memgram -c examples/COFFHeader.h -b ~/Downloads/Firefox\ Setup\ 74.0.exe -s 244 -E -e`

The `-s`(structure start offset) `-E`(reverse endian for hex view) `-e` (reverse endian for table view) are optional and can be used when displaying formatted data

## Limitations

Currentlly there is **no** support for the following:

* Non-standard C structures or arrays
* Fields size based on other field sizes