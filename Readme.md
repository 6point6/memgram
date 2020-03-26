
<h1 align="center">Memgram</h1> 
<p align="center">A CLI binary analysis tool</p>

<p align="center">
  <a href="#about">About</a> •
  <a href="#key-features">Key Features</a> •
  <a href="#grammar-format">Grammar Format</a> •
  <a href="#installation">Installation</a> •
  <a href="#how-to-use">How To Use</a> •
  <a href="#limitations">Limitations</a> 
</p>

# About

`memgram` was developed to aid with reverse engineering unknown file formats and memory structures. Custom data structures found when reverse enginneering can be quickly described in an easiliy readable toml like format called a grammar. `memgram` applies these grammars to the custom data structures in files and provides formatted prettyfied output of data.

`memgram` is inspired by the hex editors:
* [Synalyze It!](https://www.synalysis.net/)
* [010](https://www.sweetscape.com/010editor/)

## Key Features

### Grammars

`memgram` utilises easily readable file format descriptions called grammars.

<img src="https://github.com/6point6/memgram/blob/master/images/grammar_example.png" width="640" />

### Coloured Formatted Output

If certain supported types are specified, the "Formatted Data" coloumn will display the formatted data.
The current types supported are:

* hexle - Display data in little endian hex string format
* ascii - Display data in ASCII format
* ipv4be - Display data in IPv4 big endian format
* ipv4le - Display data in IPv4 little endian format
* utf16be - Display data in UTF16 big endian format
* utf16le - Display data in UTF16 little endian format
* x86_32 - Display x86_32 assembly format)

If a display type not listed above is used, `memgram` will default to formating data as a hex string in native endianess. The endianess of this default format can be changed without affecting the supported types in both the table and hex views using the -e and -E flags respectively.

<img src="https://github.com/6point6/memgram/blob/master/images/test_format.png" width="640" />

### C struct support
Support has been added for converting C structs to the grammar format with support for C basic types. Due to the fact that C basic types may have different sizes depending on the system the code is compiled on, we have picked the most common sizes (e.g short is 2 bytes). The structs can either be converted to a grammar file or used directly to display data with the option of reversing endianess.

<img src="https://github.com/6point6/memgram/blob/master/images/c_struct_example.png" width="640" />

## Grammar Format

## Installation

## How To Use

## Limitations