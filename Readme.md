
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

## Key Features

### Grammar Format

`memgram` utilises easily readable file format descriptions called grammars.

<img src="https://github.com/6point6/memgram/blob/master/images/grammar_example.png" width="640" />

### Colored formatted output

`memgram` colors outputted data in both the table view and hex view. If supported types are
specified, it also formats the data (e.g An ipv4 address).

<img src="https://github.com/6point6/memgram/blob/master/images/test_format.png" width="640" />