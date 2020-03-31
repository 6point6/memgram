
<h1 align="center">Memgram</h1> 
<p align="center">A CLI binary analysis tool</p>

<p align="center">
  <a href="#about">About</a> •
  <a href="#key-features">Key Features</a> •
  <a href="#grammar-format">Grammar Format</a> •
  <a href="#installation">Installation</a> •
  <a href="#usage-examples">Usage Examples</a> •
  <a href="#limitations">Limitations</a> 
</p>

# About

`memgram` has been developed to aid with reverse engineering unknown file formats and memory structures. Custom data structures found when reverse engineering can be quickly described in an easily readable TOML compliant format called a *grammar*. `memgram` reads a file containing a custom data structure, applies a grammar and displays formatted prettified output of data.

`memgram` is heavily inspired by the hex editors:
* [Synalyze It!](https://www.synalysis.net/)
* [010](https://www.sweetscape.com/010editor/)

## Key Features

### Grammars

`memgram` utilises easily readable file format descriptions called grammars.

<img src="https://github.com/6point6/memgram/blob/master/images/grammar_example.png" width="640" />

### Coloured Formatted Output

If a supported display type is specified in a grammar, the "Formatted Data" coloumn will display formatted data based on the display type.

Currently supported types:

* hexle - Display data in little endian hex string format
* ascii - Display data in ASCII format
* ipv4be - Display data in IPv4 big endian format
* ipv4le - Display data in IPv4 little endian format
* utf16be - Display data in UTF16 big endian format
* utf16le - Display data in UTF16 little endian format
* x86_32 - Display x86_32 assembly format)

If a display type not listed above is used, `memgram` will default to formating data as a hex string in native endianess. The endianess of this default format can be changed without affecting the supported display types in both the table and hex views when using the `-e` and `-E` flags respectively.

<img src="https://github.com/6point6/memgram/blob/master/images/test_format.png" width="640" />

### C Struct Support

C structs containing basic types can be converted to a grammar file. C basic types may have different sizes depending on the system the code is compiled on, however the most common size for each type has been selected (e.g short is 2 bytes). The structs can either be converted to a grammar file or used directly to display data with the option of reversing endianess.

<img src="https://github.com/6point6/memgram/blob/master/images/c_struct_example.png" width="640" />

### Multipliying Field Entries

To save typing, it's possible to multiply a grammar entry a specified number of times. In the example below, the field `Partition Entry` will be multiplied four times:

```toml
[metadata]
    name = 'MBR'
    variable_size_fields = [['','','','']]
    multiply_fields = [['Partition Entry','4']]

[[fields]]
    name = "Bootstrap Code"
    size = 0x1BE
    data_type = "Mixed"
    display_format = 'hex'
    description = 'MBR bootstrap code'

[[fields]]
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

### Variable Length Fields

The size of a field can be non-static and depend on other factors. For example, if the value of `variable_size_fields` is set to `[['Next Entry Offset','-','16','Filename']]` , `memgram` will set the `size` of the field called `Filename` to ((value of the data stored at `Next Entry Offset`) - 16)).

## Grammar Format

Grammars describe the data `memgram` reads, formats and displays. Grammars are written in TOML syntax.

### Metadata

Each grammar file starts with what is referred to in TOML syntax as a [Table](https://github.com/toml-lang/toml#user-content-table). The first Table in a grammar file is allways `[metadata]` and holds a several key value pairs. If uneeded, the value for each of these key pairs can be left in a default state with no information in them. For example the three fields can be left as:

```toml
[metadata]
    name = ''
    variable_size_fields = [['','','','']]
    multiply_fields = [['','']]
```

Fields can **not** be missing. For example, below is invalid syntax:

```toml
[metadata]
    name = ''
```

A default value must exist for each key. For example, the following syntax will **not** work because default values are missing for `name` and `multiply_fields`:

```toml
[metadata]
    name =
    variable_size_fields = [['','','','']]
    multiply_fields =
```

#### Name

`name` holds the name of the data structure, e.g `name = MBR`.

#### Variable Size Fields

`variable_size_fields` holds data about variable length fields. A variable length field allows for the size of a field in a grammar to be dependent on other factors instead of being a static value. 

At the moment, `memgram` supports the format:

 `[['INTEGER | SOURCE FIELD NAME', 'ARTHEMITIC | OPTION', 'INTEGER  | SOURCE FIELD NAME','VARIABLE FIELD NAME']]`

##### Format Examples

Make the size of variable field `Filename` equal to that of the value of data stored at `Next Entry Offset`:
* Set the value to `[['Next Entry Offset','','','Filename']]` or `[['','','Next Entry Offset','Filename']]`
* In this case, the `Next Entry Offset` **must** be at either position 1 or position 3 of the String array

Make the size of variable field `Filename` equal to that of ((value of data stored at `Next Entry Offset`) - 16):
* `[['Next Entry Offset','-','16','Filename']]` 
* `+, -, *, /` are all valid arithmetic operators and **must** be in position 2 of the String array.

Make the size of variable field `Filename` equal to that of (512 *  `Next Entry Offset`):
* `[['512','*','Next Entry Offset','Filename']]` 

Make the size of variable field `Filename` equal to that of first character after end of null string
* `[['','null','','Filename']]`

Multiple entries variable fields can be specified by adding another array like below:
* `[['512','*','Next Entry Offset','Filename'],['','null','','File Length']]` 

#### Multiply Fields

`multiply_fields` holds data about which fields you wish to multiply. A multiplier tells `memgram` to repeat an entry a given number of times. For example, when creating a grammar describing the `Master Boot Record` structure, instead of creating four different entries for four parition entries, we can multiply a single partition entry four times:

A multiplier is specfied in the format: `['MULTIPLIED FIELD NAME | MULTIPLIER', 'MULTIPLIED FIELD NAME | MULTIPLIER']`

For example: `[['Paritition Entry','4']]`  or  `[['4','Partition Entry']]`  are both valid. To multipliy more the one field, add annother array like so `[['4,'Partition Entry'],['2','Other Field']]`/

### Fields

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

## Installation

`memgram` can be run on the following platforms:

* Linux
* macOS
* Windows

### Standalone Binaries

Binaries for each platform can be found here: https://github.com/6point6/memgram/releases

### Compiling From Source

1. Follow instructions for installing Rust: https://www.rust-lang.org/tools/install
2. `git clone https://github.com/6point6/memgram.git`
3. `cd memgram`
4. `cargo build --release`

The binary can then be found in `./target/release/`

### Installing a Binary

1. Follow instructions for installing Rust: https://www.rust-lang.org/tools/install
2. `git clone https://github.com/6point6/memgram.git`
3. `cd memgram`
4. `cargo install --path .`

As long as `~/.cargo/bin/` is in your PATH, you should now be able to run `memgram` from the commandline.

## Usage Examples

* Display formatted data starting at offset 0 into mbr.bin based on the mbr.toml grammar:
  * `memgram -g grammar/mbr.toml -b examples/mbr.bin`
* Display description table and formatted data at offset 0 into mbr.bin based on the mbr.toml grammar:
  * `memgram -g grammar/mbr.toml -b examples/mbr.bin -d`
* Convert C struct `COFFHeader.h` to grammar file `coff_header.toml` :
  * `memgram -c examples/COFFHeader.h -o grammar/coff_header.toml`
* Use C struct COFFHeader to format data in `Firefox Setup 74.0.exe` starting at offset 244 and reverse both table and hex view endianess:
  * `memgram -c examples/COFFHeader.h -b ~/Downloads/Firefox\ Setup\ 74.0.exe -s 244 -E -e`

The `-s`(structure start offset) `-E`(reverse endian for hex view) `-e` (reverse endian for table view) are optional and can be used when displaying formatted data

## Limitations

Currentlly there is **no** support for the following:

* Non-standard C structures or arrays
